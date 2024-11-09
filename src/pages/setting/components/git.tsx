import React, { useEffect } from "react";
import { Divider, Form, Space, Button, Input, FormRule } from "antd";
import { areObjectsEqual, clearStorage, updateConfig } from "~/lib";
import { cleanConfig } from "..";
import type { MessageInstance } from "antd/es/message/interface";

const initForm: Omit<GitAuthConfig, "type"> = {
  base_url: "https://api.github.com/repos/",
  token: "",
  username: "",
  repository: "",
  path: "up2b",
};

interface GitSettingProps {
  config: Config;
  defaultConfig: Config;
  setConfig: React.Dispatch<React.SetStateAction<Config | null>>;
  setDefaultConfig: React.Dispatch<React.SetStateAction<Config | null>>;
  managerKey: GitManagerKey;
  message: MessageInstance;
}

const GitSetting = ({
  config,
  defaultConfig,
  setConfig,
  setDefaultConfig,
  managerKey,
  message,
}: GitSettingProps) => {
  const [form] = Form.useForm();

  const rules: FormRule[] = [{ required: true }];

  return (
    <Form
      form={form}
      initialValues={config.auth_config[managerKey] ?? initForm}
      onFinish={async (values) => {
        const newConfig = {
          ...config!,
          auth_config: {
            ...config.auth_config,
            [managerKey]: { ...values, type: "GIT" },
          },
        };

        cleanConfig(newConfig);

        try {
          await updateConfig(newConfig);
          setDefaultConfig(config);
          setConfig(newConfig);

          clearStorage(managerKey);

          message.success("已保存 " + managerKey + " 配置");
        } catch (e) {
          message.error(String(e));
        }
      }}
    >
      <Form.Item
        name="base_url"
        label="接口"
        rules={[...rules, { type: "url", warningOnly: true }]}
      >
        <Input disabled={managerKey === "GITHUB"} />
      </Form.Item>

      <Space wrap>
        <Form.Item name="token" label="TOKEN" rules={rules}>
          <Input.Password />
        </Form.Item>

        <Form.Item name="username" label="用户名" rules={rules}>
          <Input />
        </Form.Item>

        <Form.Item name="repository" label="仓库" rules={rules}>
          <Input />
        </Form.Item>

        <Form.Item name="path" label="目录">
          <Input placeholder="默认为 up2b" />
        </Form.Item>
      </Space>
      <Divider />

      <Form.Item
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <Space>
          <Button
            onClick={() => location.reload()}
            disabled={areObjectsEqual(defaultConfig, config)}
          >
            取消
          </Button>
          <Button type="primary" htmlType="submit">
            保存
          </Button>
        </Space>
      </Form.Item>
    </Form>
  );
};

export default GitSetting;
