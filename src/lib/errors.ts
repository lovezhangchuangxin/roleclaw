export interface ApiErrorPayload {
  code?: string;
  message?: string;
}

export function normalizeError(err: unknown): string {
  if (typeof err === "string") {
    return err;
  }

  if (err && typeof err === "object") {
    const maybe = err as { message?: string; payload?: ApiErrorPayload };

    if (maybe.payload?.code || maybe.payload?.message) {
      const code = maybe.payload.code ? `[${maybe.payload.code}] ` : "";
      return `${code}${maybe.payload.message ?? "未知错误"}`;
    }

    if (maybe.message) {
      try {
        const parsed = JSON.parse(maybe.message) as ApiErrorPayload;
        if (parsed.code || parsed.message) {
          const code = parsed.code ? `[${parsed.code}] ` : "";
          return `${code}${parsed.message ?? "未知错误"}`;
        }
      } catch {
        return maybe.message;
      }
    }
  }

  return "未知错误";
}
