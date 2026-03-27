interface Window {
  __TAURI__: {
    core: {
      invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
    };
    shell: {
      open: (url: string) => Promise<void>;
    };
    event: {
      listen: <T = unknown>(
        event: string,
        handler: (event: { payload: T }) => void,
      ) => Promise<() => void>;
    };
  };
}
