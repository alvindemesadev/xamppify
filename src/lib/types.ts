export type DiscoveryMethod = "MDns" | "PortScan" | "Manual";

export type ServiceType = "Apache" | "MySQL" | "FileZilla";

export type ServiceState = "Running" | "Stopped" | "Starting" | "Error";

export interface ServiceStatus {
  name: ServiceType;
  status: ServiceState;
  port: number;
  pid: number | null;
  uptime: number | null;
}

export interface Machine {
  id: string;
  hostname: string;
  ip: string;
  os: string | null;
  services: ServiceStatus[];
  last_seen: string;
  online: boolean;
  discovered_via: DiscoveryMethod;
}

export interface ServiceEvent {
  machine_id: string;
  service: ServiceType;
  status: ServiceState;
}

export interface LogLine {
  timestamp: string;
  level: string;
  message: string;
  source: "Apache" | "MySQL";
}

export interface FileEntry {
  name: string;
  path: string;
  size: number;
  modified: string;
  is_dir: boolean;
}

export interface QueryResult {
  columns: string[];
  rows: Record<string, unknown>[];
  affected_rows: number;
}

export interface AppHealth {
  xampp_root: string;
  xampp_available: boolean;
  openssl_available: boolean;
  apache_log_available: boolean;
  mysql_log_available: boolean;
}

export interface DiscoveryProgress {
  scanned: number;
  total: number;
}

export interface Deployment {
  name: string;
  path: string;
  url: string;
  network_url: string;
  modified: string;
  framework: string;
  tags: string[];
  pinned: boolean;
  custom_domain: string | null;
  linked_db: string | null;
  vhost_enabled: boolean;
  ssl_enabled: boolean;
  last_opened: string | null;
  has_env: boolean;
  has_composer: boolean;
  has_package_json: boolean;
  vhost_domain: string | null;
}

export type FrameworkId = "html" | "php" | "laravel" | "wordpress" | "react" | "node" | "custom";

export interface FrameworkInfo {
  id: string;
  label: string;
}

export interface BackupInfo {
  deployment: string;
  timestamp: string;
  path: string;
  size: number;
}

export interface RunOutput {
  success: boolean;
  output: string;
}

export interface GitInfo {
  is_git: boolean;
  branch: string | null;
  last_commit: string | null;
  dirty: boolean;
}

export interface VhostInfo {
  domain: string;
  root: string;
  port: number;
  ssl: boolean;
}
