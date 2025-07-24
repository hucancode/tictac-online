// Runtime configuration for API URL
// This allows the API URL to be configured at runtime instead of build time

export function getApiUrl(): string {
  // First check if there's a runtime-configured API URL
  // This could be set via a config endpoint or window object
  if (typeof window !== 'undefined' && (window as any).__API_URL__) {
    return (window as any).__API_URL__;
  }
  
  // Fall back to environment variable (build-time configuration)
  return import.meta.env.VITE_API_URL || 'http://localhost:8080';
}

// Optional: Function to set API URL at runtime
export function setApiUrl(url: string): void {
  if (typeof window !== 'undefined') {
    (window as any).__API_URL__ = url;
  }
}