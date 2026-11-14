import { useState } from "react";
import type { ScenarioQuestion } from "../types";

interface QuestionCardProps {
  question: ScenarioQuestion;
  onAnswer: (questionId: string, answer: string) => void;
  onDismiss: () => void;
}

export default function QuestionCard({
  question,
  onAnswer,
  onDismiss,
}: QuestionCardProps) {
  const [chosen, setChosen] = useState<string | null>(null);

  const handleClick = (opt: string) => {
    if (chosen) return;
    setChosen(opt);
    setTimeout(() => {
      onAnswer(question.id, opt);
    }, 450);
  };

  return (
    <div className="mx-3.5 my-2 p-3.5 rounded-2xl bg-[#0e0e12] border border-red-700/80 text-xs shadow-2xl shadow-black space-y-2.5 animate-fade-in text-neutral-100">
      <div className="flex items-center justify-between text-red-400 font-extrabold text-[11px] tracking-wider">
        <span className="flex items-center gap-1.5">
          <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
          <span>GETTING TO KNOW YOU</span>
        </span>
        <button
          onClick={onDismiss}
          className="text-neutral-400 hover:text-red-400 text-xs px-1 rounded transition-colors"
          title="Dismiss"
        >
          ✕
        </button>
      </div>

      <p className="text-white text-xs font-semibold leading-relaxed">
        {question.question}
      </p>

      <div className="space-y-1.5 pt-1">
        {question.options.map((opt) => {
          const isSelected = chosen === opt;
          return (
            <button
              key={opt}
              disabled={chosen !== null}
              onClick={() => handleClick(opt)}
              className={`
                w-full py-2 px-3 text-left rounded-xl transition-all duration-200 text-[11px] font-medium flex items-center justify-between
                ${
                  isSelected
                    ? "bg-red-600 text-white border border-red-400 shadow-md shadow-red-900/50 scale-[1.01]"
                    : chosen !== null
                    ? "opacity-40 bg-[#141418] border border-neutral-800 text-neutral-400"
                    : "bg-[#141418] hover:bg-[#201014] hover:border-red-600/80 border border-neutral-800 text-neutral-200"
                }
              `}
            >
              <span>{opt}</span>
              {isSelected && <span className="text-xs font-bold">✓ Learned</span>}
            </button>
          );
        })}
      </div>
    </div>
  );
}
