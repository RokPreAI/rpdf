import "./styles.css";
import { mountAppShell } from "./app/shell";

const appRoot = document.querySelector<HTMLElement>("#app");

if (!appRoot) {
  throw new Error("App root element was not found.");
}

mountAppShell(appRoot);
