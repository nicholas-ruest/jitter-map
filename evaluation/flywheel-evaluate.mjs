import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

const entry = process.env.METAHARNESS_FLYWHEEL_ENTRY;
if (!entry) throw new Error("METAHARNESS_FLYWHEEL_ENTRY is required");
const { makeSigner, runFlywheelGenerations, verifyReplayBundle } = await import(entry);
const repo = resolve(process.argv[2] ?? ".");
const binary = resolve(repo, "target/debug/jitter-candidate");
const frozenNoPromotion = () => ({
  promote: false,
  reasons: ["HUMAN_APPROVAL_REQUIRED", "AUTHORITY_NONE"],
});
const proposals = {
  shared_budget: "1350",
  refill_per_tick: "16",
  throttling_cost: "2",
};
const proposer = async (_base, target) => ({
  value: proposals[target],
  summary: "bounded retry-admission proposal",
});
const evaluator = async (policy) => {
  const payload = {
    variantId: "flywheel-candidate",
    genome: Object.fromEntries(
      Object.entries(policy).map(([key, value]) => [key, Number(value)]),
    ),
  };
  const child = spawnSync(binary, [], {
    encoding: "utf8",
    input: JSON.stringify(payload),
  });
  if (child.status !== 0) throw new Error(child.stderr || "candidate evaluation failed");
  const score = JSON.parse(child.stdout);
  return {
    primary: score.primary,
    noopRate: score.noopRate,
    costPerWin: score.costPerWin,
    regressed: score.regressed,
  };
};
const result = await runFlywheelGenerations({
  rootPolicy: {
    shared_budget: "1500",
    refill_per_tick: "18",
    throttling_cost: "1",
  },
  proposer,
  evaluator,
  promotionRule: frozenNoPromotion,
  holdout: {
    id: "jitter-map-frozen-v1",
    items: [{ scenario: "correlated-outage", seed: 20261004 }],
  },
  anchor: {
    id: "jitter-map-anchor-v1",
    items: [{ scenario: "correlated-outage", seed: 20261004 }],
  },
  mutationTargets: ["shared_budget", "refill_per_tick", "throttling_cost"],
  maxGenerations: 2,
  signer: makeSigner(),
  now: (generation) => `generation-${generation}`,
  cacheEvaluations: true,
  dataSource: "FROZEN_LOCAL_FIXTURE",
});
const verdict = verifyReplayBundle(result.replayBundle, {
  promotionRule: frozenNoPromotion,
});
writeFileSync(
  resolve(repo, "evidence/flywheel-replay.json"),
  JSON.stringify(result.replayBundle, null, 2) + "\n",
);
process.stdout.write(
  JSON.stringify(
    {
      package: "@metaharness/flywheel@0.1.12",
      generationsRun: result.generationsRun,
      promotions: result.promotions.length,
      replayVerdict: verdict,
      authority: "none",
    },
    null,
    2,
  ) + "\n",
);
if (!verdict.pass || result.promotions.length !== 0) process.exitCode = 1;
