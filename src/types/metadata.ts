export interface ExifData {
  [key: string]: string;
}

export interface GpuInfo {
  available: boolean;
  adapter_name: string;
  backend: string;
}

export interface AppConfig {
  proxy_resolution: number;
  thumbnail_size: number;
  cache_size_mb: number;
}
