import { afterEach } from "vitest";
import { cleanup } from "./dom";
import { clearMocks } from "./tauri";

afterEach(() => {
  cleanup();
  clearMocks();
});
