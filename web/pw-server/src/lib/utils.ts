import { ApiClient, FetchFn } from "./api_client";

export function debounce(func: Function, timeout: number = 300) {
  let timer: ReturnType<typeof setTimeout>;
  return (...args: any[]) => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      func.apply(this, args);
    }, timeout);
  };
}

export async function get(url: string, params?: Record<string, string>, fetch_fn: FetchFn = fetch) {
  const client = new ApiClient(fetch_fn);
  return await client.get(url, params);
}

export async function post(url: string, data: any, fetch_fn: FetchFn = fetch) {
  const client = new ApiClient(fetch_fn);
  return await client.post(url, data);
}
