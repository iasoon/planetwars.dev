import { get_session_token } from "./auth";

export function debounce(func: Function, timeout: number = 300) {
  let timer: ReturnType<typeof setTimeout>;
  return (...args: any[]) => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      func.apply(this, args);
    }, timeout);
  };
}

export async function get(url: string, fetch_fn: Function = fetch) {
  const headers = { "Content-Type": "application/json" };

  const token = get_session_token();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const response = await fetch_fn(url, {
    method: "GET",
    headers,
  });

  return JSON.parse(response);
}

export async function post(url: string, data: any, fetch_fn: Function = fetch) {
  const headers = { "Content-Type": "application/json" };

  const token = get_session_token();
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const response = await fetch_fn(url, {
    method: "POST",
    headers,
    body: JSON.stringify(data),
  });

  return JSON.parse(response);
}
