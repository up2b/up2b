import React from 'react'
import { Space, Form, Input, Divider, Button } from 'antd'
import { areObjectsEqual } from '~/lib'

interface CheveretoSettingProps {
  config?: Config
  defaultConfig?: Config
  setConfig: React.Dispatch<React.SetStateAction<Config | null>>
  managerKey: CheveretoManagerKey
  verifying?: boolean
  onUpdateConfig: () => void
}

const CheveretoSetting = ({
  config,
  defaultConfig,
  setConfig,
  managerKey,
  verifying,
  onUpdateConfig,
}: CheveretoSettingProps) => {
  return (
    <>
      <Space>
        <Form.Item label="用户名">
          <Input
            placeholder="输入用户名"
            value={config?.auth_config?.[managerKey]?.username || ''}
            onChange={(e) =>
              setConfig((pre) => ({
                ...pre!,
                auth_config: {
                  ...config?.auth_config,
                  [managerKey]: {
                    type: 'COMMON',
                    username: e.target.value,
                    password: config?.auth_config[managerKey]?.password || '',
                  },
                },
              }))
            }
          />
        </Form.Item>
        <Form.Item label="密码">
          <Input.Password
            placeholder="输入密码"
            value={
              config?.auth_config?.[config.using as CheveretoManagerKey]
                ?.password || ''
            }
            onChange={(e) => {
              setConfig((pre) => ({
                ...pre!,
                auth_config: {
                  ...config?.auth_config,
                  [managerKey]: {
                    type: 'CHEVERETO',
                    password: e.target.value,
                    username: config?.auth_config[managerKey]?.username || '',
                  },
                },
              }))
            }}
          />
        </Form.Item>
      </Space>

      <Divider />

      <Form.Item
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <Space>
          <Button
            onClick={() => location.reload()}
            disabled={areObjectsEqual(defaultConfig, config)}
          >
            取消
          </Button>
          <Button
            type="primary"
            loading={verifying}
            onClick={onUpdateConfig}
            disabled={
              !(config?.auth_config?.[config.using] as CheveretoAuthConfig)
                ?.username ||
              !(config?.auth_config?.[config.using] as CheveretoAuthConfig)
                ?.password ||
              areObjectsEqual(defaultConfig, config)
            }
          >
            {verifying ? '验证中...' : '保存'}
          </Button>
        </Space>
      </Form.Item>
    </>
  )
}

export default CheveretoSetting
