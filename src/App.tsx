import { HashRouter, Routes, Route } from "react-router-dom";
import { lazy, Suspense, useEffect } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "sonner";
import { AppShell } from "@/components/layout/AppShell";
import { UpdateDialog } from "@/components/updater/UpdateDialog";
import { useUpdaterStore } from "@/stores/updater-store";
import Dashboard from "@/pages/Dashboard";
import { Onboarding } from "@/components/layout/Onboarding";
import { CommandPalette } from "@/components/CommandPalette";
import { PageLoader } from "@/components/ui/PageLoader";

const ConfigEditor = lazy(() => import("@/pages/ConfigEditor"));
const DatabaseManager = lazy(() => import("@/pages/DatabaseManager"));
const SslManager = lazy(() => import("@/pages/SslManager"));
const FileBrowser = lazy(() => import("@/pages/FileBrowser"));
const Logs = lazy(() => import("@/pages/Logs"));
const Settings = lazy(() => import("@/pages/Settings"));
const FileSync = lazy(() => import("@/pages/FileSync"));
const Performance = lazy(() => import("@/pages/Performance"));
const SearchPage = lazy(() => import("@/pages/Search"));

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
  useEffect(() => {
    const timer = setTimeout(() => {
      useUpdaterStore.getState().checkNow();
    }, 10_000);
    return () => clearTimeout(timer);
  }, []);

  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <AppShell>
          <Suspense fallback={<PageLoader />}>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/logs" element={<Logs />} />
              <Route path="/files" element={<FileBrowser />} />
              <Route path="/database" element={<DatabaseManager />} />
              <Route path="/config" element={<ConfigEditor />} />
              <Route path="/ssl" element={<SslManager />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="/file-sync" element={<FileSync />} />
              <Route path="/performance" element={<Performance />} />
              <Route path="/search" element={<SearchPage />} />
            </Routes>
          </Suspense>
        </AppShell>
        <CommandPalette />
        <UpdateDialog />
      </HashRouter>
      <Toaster position="bottom-right" />
      <Onboarding />
    </QueryClientProvider>
  );
}

export default App;
