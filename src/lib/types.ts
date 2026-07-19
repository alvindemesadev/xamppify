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
}
