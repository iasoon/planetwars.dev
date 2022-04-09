import { currentUser } from "./stores/current_user";

export function set_session_token(token: string) {
  window.localStorage.setItem("session", token);
}

export function get_session_token(): string | null {
  return window.localStorage.getItem("session");
}

export function clear_session_token() {
  window.localStorage.removeItem("session");
}

export type Credentials = {
  username: string;
  password: string;
};

export async function login(credentials: Credentials) {
  let response = await fetch("/api/login", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(credentials),
  });

  if (response.status == 403) {
    throw new Error("invalid credentials");
  }
  if (!response.ok) {
    throw new Error(response.statusText);
  }

  let token = response.headers.get("Token");
  set_session_token(token);

  const user = await response.json();
  currentUser.set(user);
}
