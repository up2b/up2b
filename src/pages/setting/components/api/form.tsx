import React, { useEffect, useState } from 'react'
import { Form, Button } from 'antd'
import { getSmmsConfig } from '~/lib/api'
import ApiSetting, { initApiConfig } from '.'

interface ApiSettingProps {
  code: string
  authConfig?: ApiAuthConfig
  onChange?: (data: ApiAuthConfig) => void
}

const ApiSettingForm = ({ code, authConfig, onChange }: ApiSettingProps) => {
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
    <Form key={code} form={form} initialValues={data}>
      <ApiSetting code={code} authConfig={data} onChange={onChange} />

      <Button
        key="cancel"
        type="default"
        onClick={() => {
          // onCancel()
        }}
      >
        取消
      </Button>
      <Button key="submit" type="primary" htmlType="submit">
        确认
      </Button>
    </Form>
  )
}

export default ApiSettingForm
