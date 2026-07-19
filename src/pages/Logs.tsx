import { useState } from "react";
import { LogStream } from "@/components/log-viewer/LogStream";
import { useMachineStore } from "@/stores/machine-store";
import { useUiStore } from "@/stores/ui-store";
import { PageHeader } from "@/components/layout/PageHeader";
import { Button } from "@/components/ui/button";

export default function Logs() {
  const [unified, setUnified] = useState(false);
  const selectedMachineId = useUiStore((state) => state.selectedMachineId);
  const selectedMachine = useMachineStore((state) => state.machines.find((machine) => machine.id === selectedMachineId));
  return (
    <div className="p-6 h-full flex flex-col">
      <PageHeader
        title="Log Viewer"
        description="Search, filter, copy, and review the local Apache and MySQL logs."
        actions={
          <Button variant="outline" size="sm" onClick={() => setUnified(!unified)}>
            {unified ? "Split view" : "Unified view"}
          </Button>
        }
      />
      {selectedMachine && <p className="mb-3 rounded-lg border border-amber-500/30 bg-amber-500/10 p-2 text-xs text-amber-600 dark:text-amber-400">Logs are read from the local XAMPP installation. The active remote machine is {selectedMachine.hostname}; use its local app to inspect its files.</p>}
      <div className={`flex-1 grid gap-4 min-h-0 ${unified ? "grid-cols-1" : "grid-cols-1 lg:grid-cols-2"}`}>
        <LogStream source="Apache" />
        <LogStream source="MySQL" />
      </div>
    </div>
  );
}
