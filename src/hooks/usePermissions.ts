import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { respondPermission } from "../lib/tauri";
import type { PermissionDecision, PermissionRequest } from "../types";

export function usePermissions() {
  const [requests, setRequests] = useState<PermissionRequest[]>([]);

  useEffect(() => {
    const unlistenPromise = listen<PermissionRequest>(
      "buddy://permission-request",
      (event) => {
        setRequests((prev) => [...prev, event.payload]);
      },
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const handleRespond = async (id: string, decision: PermissionDecision) => {
    try {
      await respondPermission(id, decision);
    } finally {
      setRequests((prev) => prev.filter((r) => r.id !== id));
    }
  };

  return {
    requests,
    respond: handleRespond,
  };
}

