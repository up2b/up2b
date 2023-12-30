import React, { useEffect, useState } from 'react'
import { Form } from 'antd'
import { getSmmsConfig } from '~/lib/api'
import ApiSetting, { initApiConfig } from '.'

interface ApiSettingProps {
  code: string
  authConfig?: ApiAuthConfig
  onChange?: (data: ApiAuthConfig) => void
}

const ApiSettingForm = ({ code, authConfig, onChange }: ApiSettingProps) => {
  const [config, setConfig] = useState<ApiConfig | undefined>(authConfig?.api)

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
    }
  }, [code, config])

  if (!config) {
    return null
  }

  const data: ApiAuthConfig = {
    type: 'API',
    token: authConfig?.token ?? '',
    api: config,
  }

  return (
    <Form initialValues={data}>
      <ApiSetting code={code} authConfig={data} onChange={onChange} />
    </Form>
  )
}

export default ApiSettingForm
