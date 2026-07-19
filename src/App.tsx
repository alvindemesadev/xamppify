import { HashRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "sonner";
import { AppShell } from "@/components/layout/AppShell";
import Dashboard from "@/pages/Dashboard";
import ConfigEditor from "@/pages/ConfigEditor";
import DatabaseManager from "@/pages/DatabaseManager";
import SslManager from "@/pages/SslManager";
import FileBrowser from "@/pages/FileBrowser";
import Logs from "@/pages/Logs";
import Settings from "@/pages/Settings";
import { Onboarding } from "@/components/layout/Onboarding";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 2,
      staleTime: 30_000,
      refetchOnWindowFocus: true,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <AppShell>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/logs" element={<Logs />} />
            <Route path="/files" element={<FileBrowser />} />
            <Route path="/database" element={<DatabaseManager />} />
            <Route path="/config" element={<ConfigEditor />} />
            <Route path="/ssl" element={<SslManager />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </AppShell>
      </HashRouter>
      <Toaster position="bottom-right" />
      <Onboarding />
    </QueryClientProvider>
  );
}

export default App;
