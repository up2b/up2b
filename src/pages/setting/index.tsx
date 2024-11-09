import { useEffect, useState } from "react";
import {
  Divider,
  Form,
  Select,
  Space,
  Switch,
  Button,
  message,
  Tooltip,
} from "antd";
import { getVersion } from "@tauri-apps/api/app";
import {
  areObjectsEqual,
  getCompressState,
  getConfig,
  getImageBeds,
  updateConfig,
  verify,
} from "~/lib";
import ProxySetting from "./proxy";
import "./index.scss";
import { open } from "@tauri-apps/plugin-shell";
import { PlusOutlined } from "@ant-design/icons";
import AddCustom from "./components/add.tsx";
import ApiSettingForm from "./components/api/form.tsx";
import CheveretoSetting from "./components/chevereto.tsx";
import GitSetting from "./components/git.tsx";

/*
 * 删除非 USING 图床 keys 数量等于 1 的配置(只一个 type)
 */
export const cleanConfig = (config: Config) => {
  for (const k in config.auth_config) {
    if (
      k !== config.using &&
      Object.keys(config.auth_config[k as ManagerCode]!).length === 1
    ) {
      delete config.auth_config[k as ManagerCode];
    }
  }
};

const { Option } = Select;

interface SettingProps {
  config: Config | null;
  setConfig: React.Dispatch<React.SetStateAction<Config | null>>;
}

const Setting = ({ config, setConfig }: SettingProps) => {
  const [messageApi, contextHolder] = message.useMessage();

  const [defaultConfig, setDefaultConfig] = useState<Config | null>(config);

  const [imageBeds, setImageBeds] = useState<ManagerItem[]>([]);

  const [compressEnbled, setCompressEnbled] = useState<boolean>(true);

  const [verifying, setVerifying] = useState(false);

  const [version, setVersion] = useState<string | null>(null);

  const [index, setIndex] = useState<string | undefined>(undefined);

  const [showAddCustom, setShowAddCustom] = useState(false);

  useEffect(() => {
    if (!config) {
      getConfig().then((c) => {
        setConfig(c);
        setDefaultConfig(c);
      });
    }

    getImageBeds().then((r) => {
      setImageBeds(r);
    });

    getCompressState().then((b) => setCompressEnbled(b));
  }, []);

  useEffect(() => {
    if (!imageBeds.length) return;
    config?.using && setIndex(filterImageBed()?.index);
  }, [imageBeds, config?.using]);

  useEffect(() => {
    if (version) return;

    getVersion().then((s) => setVersion(s));
  }, []);

  const filterImageBed = () => {
    return imageBeds.find((i) => i.key === config?.using);
  };

  const onAutomaticCompressionChange = async (value: boolean) => {
    if (!config) return;

    const newConfig = { ...config, automatic_compression: value };

    try {
      await updateConfig(newConfig);
    } catch (e) {
      messageApi.error(String(e));

      return;
    }

    messageApi.success("已" + (value ? "开启" : "关闭") + "自动压缩");

    setConfig(newConfig);
    setDefaultConfig(newConfig);
  };

  const onUpdateConfig = async () => {
    if (!config) return;

    cleanConfig(config);

    let extra: Extra | null = null;

    setVerifying(true);
    try {
      extra = await verify(config.using, config.auth_config[config.using]!);
      messageApi.success("配置验证通过");
    } catch (e) {
      messageApi.error(String(e));

      return;
    } finally {
      setVerifying(false);
    }

    if (extra)
      (config.auth_config[config.using] as CheveretoAuthConfig).extra = extra;

    try {
      await updateConfig(config);
      setDefaultConfig(config);
      messageApi.success("已保存配置");
    } catch (e) {
      messageApi.error(String(e));
    }
  };

  const updateApiConfig = async (
    code: ManagerCode,
    apiAuthConfig: ApiAuthConfig,
    isNew: boolean = false,
  ) => {
    const newConfig: Config = {
      ...config!,
      using: code,
      auth_config: {
        ...config!.auth_config,
        [code]: apiAuthConfig,
      },
    };

    cleanConfig(newConfig);

    try {
      // 更新配置
      isNew || (await updateConfig(newConfig));

      setConfig(newConfig);
      setDefaultConfig(newConfig);

      // 新加图床添加到 imageBeds 中
      isNew &&
        setImageBeds((pre) => [...pre, { key: code, type: "API", name: code }]);

      messageApi.success(isNew ? "已添加 " + code : "已更新配置");
    } catch (e) {
      messageApi.error(String(e));
    }
  };

  const configKind = () => {
    const imageBedKind = filterImageBed()!.type;

    switch (imageBedKind) {
      case "API":
        const apiKey = config!.using as InferKeyType<typeof imageBedKind>;
        return (
          <ApiSettingForm
            code={apiKey}
            authConfig={config?.auth_config?.[apiKey]}
            disableCancelButton={areObjectsEqual(defaultConfig, config)}
            disableOkButton={defaultConfig?.using === apiKey}
            onOk={async (apiAuthConfig) => {
              await updateApiConfig(apiKey, apiAuthConfig);
            }}
          />
        );
      case "CHEVERETO":
        const commonKey = config!.using as InferKeyType<typeof imageBedKind>;
        return (
          <CheveretoSetting
            defaultConfig={defaultConfig!}
            config={config!}
            setConfig={setConfig}
            managerKey={commonKey}
            verifying={verifying}
            onUpdateConfig={onUpdateConfig}
          />
        );
      case "GIT":
        const key = config!.using as InferKeyType<typeof imageBedKind>;
        const form = {
          ...config!,
          auth_config: {
            ...config.auth_config,
            [key]: {
              ...config.auth_config.GITHUB!,
              type: "GIT",
              base_url: "https://api.github.com",
            },
          },
        };
        return (
          <GitSetting
            config={form}
            setConfig={setConfig}
            setDefaultConfig={setDefaultConfig}
            defaultConfig={defaultConfig!}
            managerKey={key}
            message={messageApi}
          />
        );
    }
  };

  return (
    <div id="setting">
      {contextHolder}

      <Divider style={{ padding: "0 24px" }}>基础</Divider>
      <Form.Item
        label="容灾"
        tooltip="同时将图片上传至多个图床，最大程度上避免因某个图床挂掉后图片丢失，开启此功能后图片上传的过程可能会变慢，图片的上传进度条只反映主图床的上传进度而非全部图床的上传进度"
        style={{ padding: "0 24px" }}
        extra="在完成数据库（本地或远程）缓存功能前，此功能禁用"
      >
        <Switch disabled />
      </Form.Item>

      {import.meta.env.MODE === "production" ? null : <ProxySetting />}

      {compressEnbled ? (
        <div style={{ padding: "0 24px" }}>
          <Form.Item
            label="自动压缩"
            tooltip="开启此功能后在上传图片时会自动将体积超出限制的图片自动压缩后上传图床"
          >
            <Space>
              <Switch
                checkedChildren="开启"
                unCheckedChildren="关闭"
                value={compressEnbled && config?.automatic_compression}
                onChange={onAutomaticCompressionChange}
                disabled={!config || !compressEnbled}
              />
            </Space>
          </Form.Item>
        </div>
      ) : null}

      {imageBeds.length ? (
        <div style={{ padding: "0 24px" }}>
          <Divider>图床</Divider>

          <Form.Item label="选择图床">
            <Space>
              <Select
                placeholder="选择要使用的图床"
                value={config?.using}
                onChange={(v) =>
                  setConfig((pre) =>
                    pre
                      ? { ...pre, using: v }
                      : {
                        using: v,
                        use_proxy: false,
                        automatic_compression: false,
                        auth_config: {
                          [v]: {
                            type: imageBeds.find((item) => item.key === v)
                              ?.type,
                          },
                        },
                      },
                  )
                }
                dropdownRender={(menu) => (
                  <>
                    {menu}
                    <Tooltip placement="right" title="添加自定义 api 图床">
                      <Button
                        block
                        type="text"
                        icon={<PlusOutlined />}
                        onClick={() => setShowAddCustom((pre) => !pre)}
                      ></Button>
                    </Tooltip>
                  </>
                )}
              >
                {imageBeds.map((item) => (
                  <Option key={item.key} value={item.key}>
                    {item.name}
                  </Option>
                ))}
              </Select>

              <a onClick={() => index && open(index)}>{index}</a>
            </Space>
          </Form.Item>

          {config?.using ? configKind() : null}
        </div>
      ) : null}

      <AddCustom
        show={showAddCustom}
        onOk={async (code, apiAuthConfig) => {
          await updateApiConfig(code as ManagerCode, apiAuthConfig, true);
          setShowAddCustom(false);
        }}
        onCancel={() => setShowAddCustom(false)}
      />
    </div>
  );
};

export default Setting;
