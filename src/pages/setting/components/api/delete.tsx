import React, { InputNumber } from 'antd'
import { Form, Input, Radio, Space, Switch, Select } from 'antd'
import type { FormRule } from 'antd'
import { useState } from 'react'

interface SuccessFieldProps {
  value: string | number | boolean
  disabled: boolean
  onChange: (value: string | number | boolean) => void
}

const SuccessField = ({ value, disabled, onChange }: SuccessFieldProps) => {
  const [selected, setSelected] = useState<'string' | 'number' | 'boolean'>(
    typeof value as 'string' | 'number' | 'boolean',
  )

  const render = () => {
    switch (selected) {
      case 'string':
        return <Input onChange={(e) => onChange(e.target.value)} />
      case 'number':
        return <InputNumber onChange={(v) => onChange(v!)} />
      case 'boolean':
        return (
          <Switch checked={value as boolean | undefined} onChange={onChange} />
        )
    }
  }

  return (
    <Space>
      <Select value={selected} disabled={disabled} onChange={setSelected}>
        <Select.Option value="boolean">布尔</Select.Option>
        <Select.Option value="number">数字</Select.Option>
        <Select.Option value="string">字符串</Select.Option>
      </Select>

      {render()}
    </Space>
  )
}

interface DeleteProps {
  data: ApiAuthConfig
  rules: FormRule[]
  urlRules: FormRule[]
  disabled: boolean
  handleChange: (data: ApiAuthConfig) => void
}

const Delete = ({
  data,
  rules,
  urlRules,
  disabled,
  handleChange,
}: DeleteProps) => {
  const name = (...key: string[]) => ['api', 'delete', ...key]

  const { url, method, controller } = data.api.delete

  return (
    <>
      <Form.Item label="接口" name={name('url')} rules={urlRules}>
        <Input
          placeholder="输入图片删除接口"
          value={url}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                delete: {
                  ...data.api.delete,
                  url: e.target.value,
                },
              },
            })
          }
        />
      </Form.Item>

      <Form.Item label="请求方法" name={name('method', 'type')}>
        <Radio.Group
          value={method.type}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                delete: {
                  ...data.api.delete,
                  method:
                    e.target.value === 'GET'
                      ? {
                          type: 'GET',
                          kind: (method as ApiDeleteGetMethod).kind ?? {
                            type: 'PATH',
                          },
                        }
                      : { type: 'POST' },
                },
              },
            })
          }
        >
          <Radio value="GET">GET</Radio>
          <Radio value="POST">POST</Radio>
        </Radio.Group>
      </Form.Item>

      {method.type === 'GET' ? (
        <>
          <Form.Item
            name={name('method', 'kind', 'type')}
            label="删除 id 所在位置"
          >
            <Radio.Group
              disabled={disabled}
              value={method.kind.type ?? 'PATH'}
              onChange={(e) =>
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    delete: {
                      ...data.api.delete,
                      method: {
                        ...(method as ApiDeleteGetMethod),
                        kind: {
                          ...(method as ApiDeleteGetMethod).kind,
                          type: e.target.value,
                        },
                      },
                    },
                  },
                })
              }
            >
              <Radio value="PATH">路径</Radio>
              <Radio value="QUERY">查询参数</Radio>
            </Radio.Group>
          </Form.Item>

          {method.kind.type === 'QUERY' ? (
            <Form.Item name={name('method', 'kind', 'key')} label="key">
              <Input />
            </Form.Item>
          ) : null}
        </>
      ) : null}

      <Form.Item
        name={name('controller', 'type')}
        label="是否有响应体"
        tooltip="删除图片如果有响应体则应该传入响应体中的有效属性键，否则以响应状态码判断是否成功"
        valuePropName="checked"
      >
        <Switch
          value={controller.type === 'JSON'}
          disabled={disabled}
          onChange={(v) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                delete: {
                  ...data.api.delete,
                  controller: v
                    ? {
                        ...(controller as ApiDeleteJsonController),
                        type: 'JSON',
                      }
                    : { type: 'STATUS' },
                },
              },
            })
          }
        />
      </Form.Item>

      {controller.type === 'JSON' ? (
        <Form.Item>
          <Space wrap>
            <Form.Item
              label="成功键"
              name={name('controller', 'key')}
              rules={rules}
            >
              <Input
                placeholder="删除成功与否的键名"
                value={controller.key}
                disabled={disabled}
                onChange={(e) =>
                  handleChange({
                    ...data,
                    api: {
                      ...data.api,
                      delete: {
                        ...data.api.delete,
                        controller: {
                          ...(controller as ApiDeleteJsonController),
                          key: e.target.value,
                        },
                      },
                    },
                  })
                }
              />
            </Form.Item>

            <Form.Item
              label="成功的值"
              tooltip="删除成功时值应该是什么"
              name={name('controller', 'should_be')}
              rules={rules}
            >
              <SuccessField
                value={controller.should_be}
                disabled={disabled}
                onChange={(v) =>
                  handleChange({
                    ...data,
                    api: {
                      ...data.api,
                      delete: {
                        ...data.api.delete,
                        controller: {
                          ...(controller as ApiDeleteJsonController),
                          should_be: v,
                        },
                      },
                    },
                  })
                }
              />
            </Form.Item>

            <Form.Item
              label="失败消息键"
              name={name('controller', 'message_key')}
              rules={rules}
            >
              <Input
                placeholder="删除失败的消息键名"
                value={controller.message_key}
                disabled={disabled}
                onChange={(e) =>
                  handleChange({
                    ...data,
                    api: {
                      ...data.api,
                      delete: {
                        ...data.api.delete,
                        controller: {
                          ...(controller as ApiDeleteJsonController),
                          message_key: e.target.value,
                        },
                      },
                    },
                  })
                }
              />
            </Form.Item>
          </Space>
        </Form.Item>
      ) : null}
    </>
  )
}

export default Delete
