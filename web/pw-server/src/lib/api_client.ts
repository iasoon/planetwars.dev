import { get_session_token } from "./auth";

export type FetchFn = (input: RequestInfo, init?: RequestInit) => Promise<Response>;

export class ApiError extends Error {
  constructor(public status: number, message?: string) {
    super(message);
  }
}

export class ApiClient {
  private fetch_fn: FetchFn;
  private sessionToken?: string;

  constructor(fetch_fn?: FetchFn) {
    if (fetch_fn) {
      this.fetch_fn = fetch_fn;
    } else {
      this.fetch_fn = fetch;
    }

    // TODO: maybe it is cleaner to pass this as a parameter
    this.sessionToken = get_session_token();
  }

  async get(url: string, params?: Record<string, string>): Promise<any> {
    const headers = { "Content-Type": "application/json" };

    if (this.sessionToken) {
      headers["Authorization"] = `Bearer ${this.sessionToken}`;
    }

    if (params) {
      let searchParams = new URLSearchParams(params);
      url = `${url}?${searchParams}`;
    }

    const response = await this.fetch_fn(url, {
      method: "GET",
      headers,
    });

    return await this.getJsonResponse(response);
  }

  async post(url: string, data: any): Promise<any> {
    const headers = { "Content-Type": "application/json" };

    const token = get_session_token();
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const response = await this.fetch_fn(url, {
      method: "POST",
      headers,
      body: JSON.stringify(data),
    });

    return await this.getJsonResponse(response);
  }

  private async getJsonResponse(response: Response): Promise<any> {
    if (response.ok) {
      return await response.json();
    } else {
      throw new ApiError(response.status, response.statusText);
    }
  }
}
