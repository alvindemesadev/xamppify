import { useState } from "react";
import { LogStream } from "@/components/log-viewer/LogStream";
import { useMachineStore } from "@/stores/machine-store";
import { useUiStore } from "@/stores/ui-store";
import { useQuery } from "@tanstack/react-query";
import { listDeployments } from "@/lib/ipc";
import { PageHeader } from "@/components/layout/PageHeader";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

export default function Logs() {
  const [unified, setUnified] = useState(false);
  const [deploymentFilter, setDeploymentFilter] = useState("");
  const selectedMachineId = useUiStore((state) => state.selectedMachineId);
  const selectedMachine = useMachineStore((state) => state.machines.find((machine) => machine.id === selectedMachineId));
  const { data: deployments } = useQuery({ queryKey: ["deployments"], queryFn: listDeployments });
  return (
    <div className="p-6 h-full flex flex-col">
      <PageHeader
        title="Log Viewer"
        description="Search, filter, copy, and review the local Apache and MySQL logs."
        actions={
          <div className="flex flex-wrap items-center gap-2">
            {deployments && deployments.length > 0 && (
              <Select value={deploymentFilter} onValueChange={setDeploymentFilter}>
                <SelectTrigger className="text-xs">
                  <SelectValue placeholder="All deployments" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">All deployments</SelectItem>
                  {deployments.map((d) => <SelectItem key={d.name} value={d.name}>{d.name}</SelectItem>)}
                </SelectContent>
              </Select>
            )}
            <Button variant="outline" size="sm" onClick={() => setUnified(!unified)}>
              {unified ? "Split view" : "Unified view"}
            </Button>
          </div>
        }
      />
      {selectedMachine && <p className="mb-3 rounded-lg border border-amber-500/30 bg-amber-500/10 p-2 text-xs text-amber-600 dark:text-amber-400">Logs are read from the local XAMPP installation. The active remote machine is {selectedMachine.hostname}; use its local app to inspect its files.</p>}
      {deploymentFilter && <p className="mb-3 rounded-lg border border-primary/30 bg-primary/10 p-2 text-xs text-primary">Filtering log lines that reference the <span className="font-semibold">{deploymentFilter}</span> deployment.</p>}
      <div className={`flex-1 grid gap-4 min-h-0 ${unified ? "grid-cols-1" : "grid-cols-1 lg:grid-cols-2"}`}>
        <LogStream source="Apache" deploymentFilter={deploymentFilter || undefined} />
        <LogStream source="MySQL" deploymentFilter={deploymentFilter || undefined} />
      </div>
    </div>
  );
}
