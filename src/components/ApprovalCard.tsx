import type { PermissionDecision, PermissionRequest } from "../types";

interface ApprovalCardProps {
  request: PermissionRequest;
  onRespond: (id: string, decision: PermissionDecision) => void;
}

export default function ApprovalCard({ request, onRespond }: ApprovalCardProps) {
  const isDestructive = request.risk === "destructive";

  return (
    <div
      className={`
        p-3 rounded-2xl border text-xs shadow-lg transition-all animate-fade-in
        ${
          isDestructive
            ? "bg-[#1f0a0a] border-red-700 text-red-100 shadow-black"
            : "bg-[#141418] border-amber-600/80 text-amber-100 shadow-black"
        }
      `}
    >
      <div className="flex items-center gap-1.5 font-semibold mb-1">
        <span>{isDestructive ? "⚠️ Destructive Action" : "✋ Permission Request"}</span>
        <span className="opacity-75 text-[10px] uppercase tracking-wider ml-auto font-mono text-neutral-300">
          {request.tool}
        </span>
      </div>

      <p className="leading-snug mb-2 font-mono text-[11px] break-all text-neutral-200">
        {request.description}
      </p>

      <div className="flex items-center gap-1.5 mt-2">
        <button
          onClick={() => onRespond(request.id, "approved")}
          className="
            flex-1 py-1 px-2 rounded-lg font-medium transition-colors
            bg-emerald-600 hover:bg-emerald-500 text-white
          "
        >
          Allow once
        </button>

        {!isDestructive && (
          <button
            onClick={() => onRespond(request.id, "always_allow")}
            className="
              flex-1 py-1 px-2 rounded-lg font-medium transition-colors
              bg-red-600 hover:bg-red-500 text-white
            "
          >
            Always
          </button>
        )}

        <button
          onClick={() => onRespond(request.id, "denied")}
          className="
            py-1 px-2.5 rounded-lg font-medium transition-colors
            bg-neutral-800 hover:bg-neutral-700 text-neutral-200
          "
        >
          Deny
        </button>
      </div>
    </div>
  );
}
