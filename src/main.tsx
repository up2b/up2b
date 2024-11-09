import React from "react";
import ReactDOM from "react-dom/client";
import { ConfigProvider, theme } from "antd";
import zhCN from "antd/locale/zh_CN";
import App from "./App";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import "./styles.scss";
const appWindow = getCurrentWebviewWindow();

if (import.meta.env.MODE === "production") {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
}

appWindow.theme().then((v) => {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ConfigProvider
        locale={zhCN}
        theme={{
          cssVar: true,
          algorithm:
            v === "dark" ? theme.darkAlgorithm : theme.defaultAlgorithm,
        }}
      >
        <App />
      </ConfigProvider>
    </React.StrictMode>,
  );
});
