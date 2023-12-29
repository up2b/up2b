import React from 'antd'
import { Form, Input, Radio, Space, Switch, Select } from 'antd'
import type { FormRule } from 'antd'

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

  return (
    <>
      <Form.Item label="接口" name={name('url')} rules={urlRules}>
        <Input
          placeholder="输入图片删除接口"
          value={data.api.delete.url}
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
          value={data.api.delete.method.type}
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
                        kind: (data.api.delete.method as ApiDeleteGetMethod)
                          .kind ?? { type: 'PATH' },
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

      {data.api.delete.method.type === 'GET' ? (
        <>
          <Form.Item
            name={name('method', 'kind', 'type')}
            label="删除 id 所在位置"
          >
            <Radio.Group
              disabled={disabled}
              value={data.api.delete.method.kind.type ?? 'PATH'}
              onChange={(e) =>
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    delete: {
                      ...data.api.delete,
                      method: {
                        ...(data.api.delete.method as ApiDeleteGetMethod),
                        kind: {
                          ...(data.api.delete.method as ApiDeleteGetMethod)
                            .kind,
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

          {data.api.delete.method.kind.type === 'QUERY' ? (
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
          value={data.api.delete.controller.type === 'JSON'}
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
                      ...(data.api.delete
                        .controller as ApiDeleteJsonController),
                      type: 'JSON',
                    }
                    : { type: 'STATUS' },
                },
              },
            })
          }
        />
      </Form.Item>

      {data.api.delete.controller.type === 'JSON' ? (
        <Form.Item>
          <Space wrap>
            <Form.Item
              label="成功键"
              name={name('controller', 'key')}
              rules={rules}
            >
              <Input
                placeholder="删除成功与否的键名"
                value={data.api.delete.controller.key}
                disabled={disabled}
                onChange={(e) =>
                  handleChange({
                    ...data,
                    api: {
                      ...data.api,
                      delete: {
                        ...data.api.delete,
                        controller: {
                          ...(data.api.delete
                            .controller as ApiDeleteJsonController),
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
              <Input
                addonBefore={
                  <Select
                    value={typeof data.api.delete.controller.should_be}
                    disabled={disabled}
                  >
                    <Select.Option value="boolean">布尔</Select.Option>
                    <Select.Option value="number">数字</Select.Option>
                    <Select.Option value="string">字符串</Select.Option>
                  </Select>
                }
                placeholder="删除成功的值"
                value={data.api.delete.controller.should_be}
                disabled={disabled}
                onChange={(e) =>
                  handleChange({
                    ...data,
                    api: {
                      ...data.api,
                      delete: {
                        ...data.api.delete,
                        controller: {
                          ...(data.api.delete
                            .controller as ApiDeleteJsonController),
                          // TODO: 根据 addonBefore 的数据类型进行转换
                          should_be: e.target.value,
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
                value={data.api.delete.controller.message_key}
                disabled={disabled}
                onChange={(e) =>
                  handleChange({
                    ...data,
                    api: {
                      ...data.api,
                      delete: {
                        ...data.api.delete,
                        controller: {
                          ...(data.api.delete
                            .controller as ApiDeleteJsonController),
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
