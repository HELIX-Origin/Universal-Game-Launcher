import en from "./locales/en";

type MessagePaths<T> = {
  [K in keyof T & string]: T[K] extends string
    ? K
    : T[K] extends Record<string, unknown>
      ? `${K}.${MessagePaths<T[K]>}`
      : never;
}[keyof T & string];

export type MessageKey = MessagePaths<typeof en>;

export function t(key: MessageKey, params: Record<string, string | number> = {}): string {
  let message: unknown = en;
  for (const part of key.split(".")) {
    if (typeof message !== "object" || message === null || !(part in message)) {
      return key;
    }
    message = (message as Record<string, unknown>)[part];
  }

  if (typeof message !== "string") return key;
  return message.replace(/\{([^}]+)\}/g, (placeholder, name: string) =>
    name in params ? String(params[name]) : placeholder,
  );
}
