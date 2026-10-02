/**
 * Minimal DOM utilities for component tests (jsdom project).
 * Queries use accessible names (aria-label, otherwise trimmed text) so tests
 * follow what a user or assistive technology perceives.
 */
import { flushSync, mount, tick, unmount, type Component } from "svelte";

const mounted: Record<string, unknown>[] = [];

export function render(component: Component, props: Record<string, unknown> = {}) {
  const target = document.createElement("div");
  document.body.appendChild(target);
  const instance = mount(component, { target, props });
  mounted.push(instance);
  flushSync();
  return { target };
}

export function cleanup() {
  for (const instance of mounted.splice(0)) unmount(instance);
  document.body.innerHTML = "";
}

/** Let pending IPC promises and Svelte updates settle. */
export async function settle(rounds = 5) {
  for (let i = 0; i < rounds; i += 1) {
    await new Promise((resolve) => setTimeout(resolve, 0));
    await tick();
  }
  flushSync();
}

export async function waitFor<T>(check: () => T, timeoutMs = 1000): Promise<T> {
  const started = Date.now();
  let lastError: unknown;
  while (Date.now() - started < timeoutMs) {
    try {
      return check();
    } catch (error) {
      lastError = error;
      await settle(1);
    }
  }
  throw lastError;
}

/** Text content excluding `aria-hidden` decoration (icons, spinners). */
export function visibleText(element: Node): string {
  let text = "";
  for (const child of element.childNodes) {
    if (child.nodeType === Node.TEXT_NODE) text += child.textContent ?? "";
    else if (child instanceof Element && child.getAttribute("aria-hidden") !== "true") {
      text += visibleText(child);
    }
  }
  return text.replace(/\s+/g, " ").trim();
}

export function accessibleName(element: Element): string {
  const label = element.getAttribute("aria-label");
  if (label) return label.trim();
  const labelledBy = element.getAttribute("aria-labelledby");
  if (labelledBy) {
    const text = document.getElementById(labelledBy)?.textContent;
    if (text) return text.replace(/\s+/g, " ").trim();
  }
  return visibleText(element);
}

function matches(name: string, expected: string | RegExp) {
  return typeof expected === "string" ? name === expected : expected.test(name);
}

export function queryButtons(name: string | RegExp, root: ParentNode = document): HTMLButtonElement[] {
  return [...root.querySelectorAll("button")].filter((button) =>
    matches(accessibleName(button), name),
  );
}

export function getButton(name: string | RegExp, root: ParentNode = document): HTMLButtonElement {
  const found = queryButtons(name, root);
  if (found.length !== 1) {
    const available = [...root.querySelectorAll("button")].map(accessibleName).join(" | ");
    throw new Error(`Expected one button named ${name}, found ${found.length}. Buttons: ${available}`);
  }
  return found[0];
}

/** Finds a form control by the visible text of its wrapping `<label>`. */
export function getField<T extends HTMLElement = HTMLInputElement>(
  labelText: string | RegExp,
  root: ParentNode = document,
): T {
  for (const label of root.querySelectorAll("label")) {
    const control = label.querySelector("input, select, textarea");
    if (!control) continue;
    const span = label.querySelector("span");
    const names = [
      span?.querySelector("strong")?.textContent,
      span?.textContent,
      label.textContent,
    ].map((name) => (name ?? "").replace(/\s+/g, " ").trim());
    if (names.some((name) => name && matches(name, labelText))) return control as unknown as T;
  }
  throw new Error(`No form field labelled ${labelText}`);
}

export function getDialog(title: string | RegExp): HTMLDialogElement {
  const dialog = [...document.querySelectorAll("dialog")].find((element) =>
    matches(accessibleName(element), title),
  );
  if (!dialog) throw new Error(`No dialog titled ${title}`);
  return dialog;
}

export function textOf(root: ParentNode = document.body): string {
  return (root.textContent ?? "").replace(/\s+/g, " ").trim();
}

export function gameTitles(): string[] {
  return [...document.querySelectorAll(".game-card h2")].map((heading) => textOf(heading));
}

export function gameCard(title: string): HTMLElement {
  const card = [...document.querySelectorAll<HTMLElement>(".game-card")].find(
    (element) => textOf(element.querySelector("h2") ?? element) === title,
  );
  if (!card) throw new Error(`No game card titled ${title}. Cards: ${gameTitles().join(" | ")}`);
  return card;
}

export function alerts(): string[] {
  return [...document.querySelectorAll('[role="alert"]')].map((element) => textOf(element));
}

export async function click(element: HTMLElement) {
  element.click();
  await settle();
}

export async function typeInto(element: HTMLInputElement | HTMLSelectElement, value: string) {
  element.value = value;
  element.dispatchEvent(new Event(element instanceof HTMLSelectElement ? "change" : "input", { bubbles: true }));
  await settle();
}

export async function toggle(checkbox: HTMLInputElement) {
  checkbox.click();
  await settle();
}

export async function submit(form: HTMLFormElement) {
  form.requestSubmit();
  await settle();
}
