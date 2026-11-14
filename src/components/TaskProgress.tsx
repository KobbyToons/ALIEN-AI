import type { AgentStepEvent } from "../types";

interface TaskProgressProps {
  steps: AgentStepEvent[];
}

export default function TaskProgress({ steps }: TaskProgressProps) {
  if (steps.length === 0) return null;

  return (
    <div className="mx-3.5 my-1.5 p-3 rounded-2xl bg-[#141418] border border-red-800/60 text-xs shadow-xl shadow-black space-y-2 animate-fade-in">
      <div className="flex items-center justify-between font-semibold text-red-400">
        <span className="flex items-center gap-1.5">
          <span className="inline-block w-2 h-2 rounded-full bg-red-500 animate-ping" />
          Autonomous Task
        </span>
        <span className="text-[10px] font-mono text-neutral-300">
          Step {steps[steps.length - 1]?.step || 1}
        </span>
      </div>

      <div className="space-y-1.5 max-h-[110px] overflow-y-auto chat-scroll pr-1 font-mono text-[11px]">
        {steps.map((st, i) => (
          <div key={i} className="flex items-start gap-2">
            <span className="text-[10px] text-neutral-300">
              {st.status === "done"
                ? "✓"
                : st.status === "error"
                ? "✗"
                : "⚙"}
            </span>
            <span
              className={`leading-tight ${
                st.status === "executing"
                  ? "text-red-400 font-bold"
                  : "text-neutral-200"
              }`}
            >
              {st.description}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
