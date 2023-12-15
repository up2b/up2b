import React, { Form } from 'antd'
import { Switch, Space, Input, InputNumber, Button, Select } from 'antd'
import { CheckOutlined } from '@ant-design/icons'
import { ChangeEvent, useState } from 'react'

const { Option } = Select

interface ProxySettingProps {
  value?: Proxy
}

const ProxySetting = ({ value }: ProxySettingProps) => {
  const [useProxy, setUseProxy] = useState(false)

  const [protocol, setProtocol] = useState<ProxyProtocol | undefined>(
    value?.type,
  )
  const [host, setHost] = useState<string | undefined>(value?.host)
  const [port, setPort] = useState<number | null>(value?.port ?? null)

  const onProxyProtocolChange = (v: ProxyProtocol) => {
    setProtocol(v)
  }
  const onProxyHostChange = (e: ChangeEvent<HTMLInputElement>) => {
    if (!e.target.value) return

    setHost(e.target.value)
  }
  const onProxyPortChange = (v: number | null) => {
    setPort(v)
  }

  const onFinish = (value: any) => {
    console.log(value)
  }

  return (
    <Form onFinish={onFinish} style={{ padding: '0 24px' }}>
      <Space wrap>
        <Space.Compact style={{ margin: 0 }}>
          <Form.Item label="代理" style={{ margin: 0 }}>
            <Switch
              checkedChildren="开启"
              unCheckedChildren="关闭"
              checked={useProxy}
              onChange={(v) => setUseProxy(v)}
              style={{ marginRight: 5 }}
            />
          </Form.Item>

          {useProxy ? (
            <Form.Item name="proxy" style={{ margin: 0 }}>
              <Space direction="vertical" size="middle">
                <Space.Compact>
                  <Form.Item
                    name={['proxy', 'type']}
                    rules={[{ required: true, message: '请选择协议' }]}
                    style={{ margin: 0 }}
                  >
                    <Select
                      placeholder="选择协议"
                      defaultValue={protocol}
                      onChange={onProxyProtocolChange}
                    >
                      <Option value="http">http://</Option>
                      <Option value="https">https://</Option>
                      <Option value="socks5">socks5://</Option>
                      <Option value="socks5h">socks5h://</Option>
                    </Select>
                  </Form.Item>
                  <Form.Item
                    name={['proxy', 'host']}
                    rules={[{ required: true, message: '请输入 host' }]}
                    style={{ margin: 0 }}
                  >
                    <Input
                      placeholder="代理主机地址，如 127.0.0.1"
                      value={host}
                      onChange={onProxyHostChange}
                    />
                  </Form.Item>
                  <Form.Item
                    name={['proxy', 'port']}
                    rules={[{ required: true, message: '请输入 port' }]}
                    style={{ margin: 0 }}
                  >
                    <InputNumber
                      min={1}
                      max={65536}
                      placeholder="端口"
                      value={port}
                      onChange={onProxyPortChange}
                    />
                  </Form.Item>
                  <Button
                    type="primary"
                    icon={<CheckOutlined />}
                    htmlType="submit"
                  />
                </Space.Compact>
              </Space>
            </Form.Item>
          ) : null}
        </Space.Compact>
      </Space>
    </Form>
  )
}

export default ProxySetting
