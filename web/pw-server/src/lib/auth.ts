export function set_session_token(token: string) {
  window.localStorage.setItem("session", token);
}

export function get_session_token(): string | null {
  return window.localStorage.getItem("session");
}
