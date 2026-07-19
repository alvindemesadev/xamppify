import { invoke } from "@tauri-apps/api/core";
import type { AppHealth, Deployment, Machine, ServiceStatus, FileEntry, QueryResult, LogLine } from "./types";

export const startDiscovery = () => invoke<void>("start_discovery");
export const stopDiscovery = () => invoke<void>("stop_discovery");
export const getDiscoveredMachines = () => invoke<Machine[]>("get_discovered_machines");
export const getXamppRoot = () => invoke<string>("get_xampp_root");
export const getAppHealth = () => invoke<AppHealth>("get_app_health");
export const addManualMachine = (ip: string) => invoke<Machine>("add_manual_machine", { ip });

export const listDeployments = () => invoke<Deployment[]>("list_deployments");
export const createDeployment = (name: string, template: "html" | "php") => invoke<Deployment>("create_deployment", { name, template });
export const importDeployment = (name: string, sourcePath: string) => invoke<Deployment>("import_deployment", { name, sourcePath });
export const deleteDeployment = (name: string) => invoke<void>("delete_deployment", { name });

export const startService = (machineId: string, service: string) =>
  invoke<void>("start_service", { machineId, service });
export const stopService = (machineId: string, service: string) =>
  invoke<void>("stop_service", { machineId, service });
export const restartService = (machineId: string, service: string) =>
  invoke<void>("restart_service", { machineId, service });
export const getServiceStatus = (machineId: string) =>
  invoke<ServiceStatus[]>("get_service_status", { machineId });

export const listDirectory = (dir: string) => invoke<FileEntry[]>("list_directory", { dir });
export const readFile = (path: string) => invoke<string>("read_file", { path });
export const writeFile = (path: string, content: string) =>
  invoke<void>("write_file", { path, content });
export const createFolder = (parent: string, name: string) => invoke<void>("create_folder", { parent, name });
export const createFile = (parent: string, name: string) => invoke<string>("create_file", { parent, name });
export const deletePath = (path: string) => invoke<void>("delete_path", { path });
export const renamePath = (path: string, newName: string) => invoke<string>("rename_path", { path, newName });
export const uploadFiles = (destination: string, sourcePaths: string[]) => invoke<void>("upload_files", { destination, sourcePaths });
export const uploadFolder = (destination: string, sourcePath: string) => invoke<void>("upload_folder", { destination, sourcePath });

export const mysqlConnect = (host: string | null, port: number | null, user: string, password: string) =>
  invoke<void>("mysql_connect", { host, port, user, password });
export const mysqlDisconnect = () => invoke<void>("mysql_disconnect");
export const listDatabases = () => invoke<string[]>("list_databases");
export const listTables = (database: string) =>
  invoke<string[]>("list_tables", { database });
export const runQuery = (query: string) =>
  invoke<QueryResult>("run_query", { query });
export const exportDatabase = (database: string) =>
  invoke<string>("export_database", { database });

export const getLogs = (source: string, maxLines?: number) =>
  invoke<LogLine[]>("get_logs", { source, maxLines });
export const startLogWatcher = (source: string) =>
  invoke<void>("start_log_watcher", { source });

export const getKnownConfigs = () =>
  invoke<{ name: string; path: string; category: string }[]>("get_known_configs");
export const parseIniSections = (content: string) =>
  invoke<{ name: string; line: number }[]>("parse_ini_sections", { content });

export const listCertificates = () =>
  invoke<{ name: string; path: string; is_key: boolean }[]>("list_certificates");
export const readCertificate = (path: string) =>
  invoke<{
    name: string;
    cert_path: string;
    key_path: string | null;
    subject: string;
    issuer: string;
    valid_from: string;
    valid_to: string;
    san: string[];
  }>("read_certificate", { path });
export const syncToRemote = (source: string, destination: string, remoteHost: string) =>
  invoke<{ success: boolean; output: string; files_copied: number }>("sync_to_remote", { source, destination, remoteHost });
export const getLocalPerformance = () =>
  invoke<{ cpu_percent: number; memory_percent: number; memory_used_gb: number; memory_total_gb: number; disk_percent: number; disk_free_gb: number; disk_total_gb: number; uptime_days: number }>("get_local_performance");
export const generateSelfSigned = (commonName: string, days: number) =>
  invoke<{
    name: string;
    cert_path: string;
    key_path: string | null;
    subject: string;
    issuer: string;
    valid_from: string;
    valid_to: string;
    san: string[];
  }>("generate_self_signed", { commonName, days });
