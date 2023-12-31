import React, { useEffect, useState } from 'react'
import { Form, Button, Space } from 'antd'
import { getSmmsConfig } from '~/lib/api'
import ApiSetting, { initApiConfig } from '.'

interface ApiSettingProps {
  code: string
  authConfig?: ApiAuthConfig
  onChange?: (data: ApiAuthConfig) => void
  onOk?: () => void
  disableCancelButton?: boolean
  disableOkButton?: boolean
}

const ApiSettingForm = ({
  code,
  authConfig,
  onChange,
  onOk,
  disableCancelButton,
  disableOkButton,
}: ApiSettingProps) => {
  const [form] = Form.useForm()

  const [config, setConfig] = useState<ApiConfig | undefined>(authConfig?.api)

  useEffect(() => {
    if (config) {
      form.resetFields()
    }
  }, [code, config])

  useEffect(() => {
    if (!authConfig?.api) {
      if (code === 'SMMS') {
        // 第一次配置 smms 时才会获取 smms 示例配置
        getSmmsConfig().then((c) => {
          setConfig(c)
        })
      } else {
        setConfig(initApiConfig.api)
      }
    } else {
      setConfig(authConfig.api)
    }
  }, [code, authConfig])

  if (!config) {
    return null
  }

  const data: ApiAuthConfig = {
    type: 'API',
    token: authConfig?.token ?? '',
    api: config,
  }

  return (
    <Form form={form} initialValues={data} onFinish={onOk}>
      <ApiSetting code={code} authConfig={data} onChange={onChange} />

      <Form.Item
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          marginBottom: '2rem',
        }}
      >
        <Space>
          <Button
            key="cancel"
            type="default"
            onClick={() => location.reload()}
            disabled={disableCancelButton}
          >
            取消
          </Button>
          <Button
            key="submit"
            type="primary"
            htmlType="submit"
            disabled={disableOkButton}
          >
            保存
          </Button>
        </Space>
      </Form.Item>
    </Form>
  )
}

export default ApiSettingForm
