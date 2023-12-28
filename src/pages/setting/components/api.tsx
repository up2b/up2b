import React, { useEffect, useState } from 'react'
import {
  Divider,
  Form,
  Input,
  InputNumber,
  Radio,
  Select,
  Space,
  Switch,
} from 'antd'
import { getSmmsConfig } from '~/lib/api'

interface ApiSettingProps {
  code: string
  token?: string
  config?: ApiConfig
  onChange: (data: ApiAuthConfig) => void
}

const ALLOWED_FORMATS = ['PNG', 'JPEG', 'GIF', 'WEBP', 'BMP']

const initApiConfig: ApiConfig = {
  auth_method: {
    type: 'HEADER',
  },
  list: {
    url: '',
    method: { type: 'GET' },
    controller: {
      items_key: '',
      image_url_key: '',
      deleted_id_key: '',
    },
  },
  delete: {
    url: '',
    method: { type: 'GET', kind: { type: 'PATH' } },
    controller: { type: 'JSON', key: '', should_be: '' },
  },
  upload: {
    url: '',
    max_size: 0,
    timeout: 0,
    allowed_formats: ['PNG', 'JPEG', 'GIF'],
    content_type: {
      type: 'MULTIPART',
      file_kind: 'STREAM',
      file_part_name: '',
    },
    controller: {
      image_url_key: '',
      deleted_id_key: '',
    },
  },
}

const ApiSetting = ({ code, token, config, onChange }: ApiSettingProps) => {
  console.log(code, config)
  const [data, setData] = useState<ApiAuthConfig>({
    type: 'API',
    token: token ?? '',
    api: config ?? initApiConfig,
  })

  useEffect(() => {
    if (code === 'SMMS' && !config) {
      getSmmsConfig().then((c) => {
        setData({ ...data, api: c })
      })
    }
  }, [code, config])

  const filteredFormats = ALLOWED_FORMATS.filter(
    (o) => !config?.upload.allowed_formats.includes(o),
  )

  const disabled = code === 'SMMS'

  const handleChange = (data: ApiAuthConfig) => {
    setData(data)

    onChange(data)
  }

  console.log(data)

  return (
    <>
      <Form.Item name="token" label="TOKEN" rules={[{ required: true }]}>
        <Input.Password
          placeholder="输入 token"
          value={data.token || ''}
          onChange={(e) => handleChange({ ...data, token: e.target.value })}
        />
      </Form.Item>

      <Divider>认证方式</Divider>

      <Form.Item
        label="token 所在的位置"
        tooltip="请求体只能是 json 类型"
        rules={[{ required: true }]}
      >
        <Radio.Group
          value={data.api.auth_method.type}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                auth_method:
                  e.target.value === 'HEADER'
                    ? {
                        ...(data.api.auth_method as AuthHeaderMethod),
                        type: 'HEADER',
                      }
                    : {
                        ...(data.api.auth_method as AuthBodyMethod),
                        type: 'BODY',
                      },
              },
            })
          }
        >
          <Radio value="HEADER">请求头</Radio>
          <Radio value="BODY">请求体</Radio>
        </Radio.Group>
      </Form.Item>

      {data.api.auth_method.type === 'HEADER' ? (
        <Space>
          <Form.Item label="key">
            <Input
              value={data.api.auth_method.key ?? 'Authorization'}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    auth_method: {
                      ...(data.api.auth_method as AuthHeaderMethod),
                      key: e.target.value,
                    },
                  },
                })
              }
            />
          </Form.Item>

          <Form.Item
            label="前缀"
            tooltip="token 前的前缀，不可省略空格，如果不需要可不填此项"
          >
            <Input
              value={data.api.auth_method.prefix}
              placeholder="token 前缀"
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    auth_method: {
                      ...(data.api.auth_method as AuthHeaderMethod),
                      prefix: e.target.value,
                    },
                  },
                })
              }
            />
          </Form.Item>
        </Space>
      ) : (
        <Form.Item
          name={['data', 'api', 'auth_method', 'key']}
          label="key"
          rules={[{ required: true }]}
        >
          <Input
            value={data.api.auth_method.key}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  auth_method: {
                    ...(data.api.auth_method as AuthHeaderMethod),
                    key: e.target.value,
                  },
                },
              })
            }
          />
        </Form.Item>
      )}

      <Divider>图片列表</Divider>

      <Form.Item
        name={['data', 'api', 'list', 'url']}
        label="接口"
        rules={[{ required: true }]}
      >
        <Input
          placeholder="输入图片列表接口"
          value={data.api.list.url}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                list: { ...data.api.list, url: e.target.value },
              },
            })
          }
        />
      </Form.Item>

      <Form.Item label="请求方法" rules={[{ required: true }]}>
        <Radio.Group
          value={data.api.list.method.type}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              api: {
                ...data.api,
                list: {
                  ...data.api.list,
                  method: { ...data.api.list.method, type: e.target.value },
                },
              },
            })
          }
        >
          <Radio value="GET">GET</Radio>
          <Radio value="POST">POST</Radio>
        </Radio.Group>
      </Form.Item>

      <Form.Item name={['data', 'api', 'list', 'controller']}>
        <Space wrap>
          <Form.Item
            label="图片数组键"
            name={['data', 'api', 'list', 'controller', 'items_key']}
            rules={[{ required: true }]}
          >
            <Input
              placeholder="输入图片数组键名"
              value={data.api.list.controller.items_key}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    list: {
                      ...data.api.list,
                      controller: {
                        ...data.api.list.controller,
                        items_key: e.target.value,
                      },
                    },
                  },
                })
              }
            />
          </Form.Item>

          <Form.Item
            label="图片地址键"
            name={['data', 'api', 'list', 'controller', 'image_url_key']}
            rules={[{ required: true }]}
          >
            <Input
              placeholder="输入图片地址键名"
              value={data.api.list.controller.image_url_key}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    list: {
                      ...data.api.list,
                      controller: {
                        ...data.api.list.controller,
                        image_url_key: e.target.value,
                      },
                    },
                  },
                })
              }
            />
          </Form.Item>

          <Form.Item
            label="图片删除 id 键"
            name={['data', 'api', 'list', 'controller', 'deleted_id_key']}
            rules={[{ required: true }]}
          >
            <Input
              placeholder="输入图片删除 id 键名"
              value={data.api.list.controller.deleted_id_key}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    list: {
                      ...data.api.list,
                      controller: {
                        ...data.api.list.controller,
                        deleted_id_key: e.target.value,
                      },
                    },
                  },
                })
              }
            />
          </Form.Item>

          <Form.Item label="图片缓存键">
            <Input
              placeholder="输入图片缓存键名"
              value={data.api.list.controller.thumb_key}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    list: {
                      ...data.api.list,
                      controller: {
                        ...data.api.list.controller,
                        thumb_key: e.target.value,
                      },
                    },
                  },
                })
              }
            />
          </Form.Item>
        </Space>
      </Form.Item>

      <Divider>删除</Divider>

      <Form.Item
        label="接口"
        name={['data', 'api', 'delete', 'url']}
        rules={[{ required: true }]}
      >
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

      <Form.Item label="请求方法">
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
                          kind:
                            (data.api.delete.method as ApiDeleteGetMethod)
                              .kind ??
                            (initApiConfig.delete.method as ApiDeleteGetMethod)
                              .kind,
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
          <Form.Item label="删除 id 所在位置">
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
            <Form.Item label="key">
              <Input />
            </Form.Item>
          ) : null}
        </>
      ) : null}

      <Form.Item
        label="是否有响应体"
        tooltip="删除图片如果有响应体则应该传入响应体中的有效属性键，否则以响应状态码判断是否成功"
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
          <Space>
            <Form.Item
              label="成功键"
              name={['data', 'api', 'delete', 'controller', 'key']}
              rules={[{ required: true }]}
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
              name={['data', 'api', 'delete', 'controller', 'should_be']}
              rules={[{ required: true }]}
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
              name={['data', 'api', 'delete', 'controller', 'message_key']}
              rules={[{ required: true }]}
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

      <Divider>上传</Divider>

      <Form.Item>
        <Form.Item
          label="接口"
          name={['data', 'api', 'upload', 'url']}
          rules={[{ required: true }]}
        >
          <Input
            placeholder="输入上传图片接口"
            value={data.api.upload.url}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  upload: {
                    ...data.api.upload,
                    url: e.target.value,
                  },
                },
              })
            }
          />
        </Form.Item>

        <Form.Item
          label="最大体积"
          name={['data', 'api', 'upload', 'max_size']}
          rules={[{ required: true }]}
        >
          <InputNumber
            placeholder="输入允许的最大体积"
            value={
              data.api.upload.max_size
                ? data.api.upload.max_size / 1024 / 1024
                : undefined
            }
            disabled={disabled}
            addonAfter="MB"
            onChange={(v) =>
              v &&
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  upload: {
                    ...data.api.upload,
                    max_size: v,
                  },
                },
              })
            }
          />
        </Form.Item>

        <Form.Item
          label="允许的格式"
          name={['data', 'api', 'upload', 'allowed_formats']}
          rules={[{ required: true }]}
        >
          <Select
            mode="multiple"
            placeholder="选择图片格式"
            value={config?.upload.allowed_formats ?? []}
            onChange={(values) => {
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  upload: {
                    ...data.api.upload,
                    allowed_formats: values,
                  },
                },
              })
            }}
            options={filteredFormats.map((item) => ({
              value: item,
              label: item,
            }))}
            disabled={disabled}
          />
        </Form.Item>

        <Form.Item label="请求体类型">
          <Radio.Group
            value={data.api.upload.content_type.type}
            disabled={disabled}
            onChange={(e) => {
              handleChange({
                ...data,
                api: {
                  ...data.api,
                  upload: {
                    ...data.api.upload,
                    content_type:
                      e.target.value === 'JSON'
                        ? {
                            ...(data.api.upload
                              .content_type as ApiUploadJsonContentType),
                            type: 'JSON',
                          }
                        : {
                            ...(data.api.upload
                              .content_type as ApiUploadMultipartContentType),
                            type: 'MULTIPART',
                          },
                  },
                },
              })
            }}
          >
            <Radio value="MULTIPART">multipart</Radio>
            <Radio value="JSON">json</Radio>
          </Radio.Group>
        </Form.Item>

        {data.api.upload.content_type.type === 'MULTIPART' ? (
          <Space>
            <Form.Item
              label="上传类型"
              tooltip="流式响应支持上传进度，流式上传失败时可尝试更换为 bytes"
            >
              <Radio.Group
                value={data.api.upload.content_type.file_kind ?? 'STREAM'}
                disabled={disabled}
                onChange={(e) => {
                  handleChange({
                    ...data,
                    api: {
                      ...data.api,
                      upload: {
                        ...data.api.upload,
                        content_type: {
                          ...(data.api.upload
                            .content_type as ApiUploadMultipartContentType),
                          file_kind: e.target.value,
                        },
                      },
                    },
                  })
                }}
              >
                <Radio value="STREAM">流</Radio>
                <Radio value="BUFFER">bytes</Radio>
              </Radio.Group>
            </Form.Item>

            <Form.Item
              label="图片的表单键"
              name={['data', 'api', 'upload', 'content_type', 'file_part_name']}
              rules={[{ required: true }]}
            >
              <Input
                value={data.api.upload.content_type.file_part_name}
                disabled={disabled}
                onChange={(e) => {
                  handleChange({
                    ...data,
                    api: {
                      ...data.api,
                      upload: {
                        ...data.api.upload,
                        content_type: {
                          ...(data.api.upload
                            .content_type as ApiUploadMultipartContentType),
                          file_part_name: e.target.value,
                        },
                      },
                    },
                  })
                }}
              />
            </Form.Item>
          </Space>
        ) : (
          <>
            {/*TODO: 以后再完善 json 上传*/}
            <Form.Item
              label="图片数组的表单键"
              name={['data', 'api', 'upload', 'content_type', 'key']}
              rules={[{ required: true }]}
            >
              <Input value={data.api.upload.content_type.key} />
            </Form.Item>

            <Form.Item label="除图片之外的其他 json 数据">
              <Input.TextArea />
            </Form.Item>
          </>
        )}

        <Space>
          <Form.Item
            label="图片键"
            name={['data', 'api', 'upload', 'controller', 'image_url_key']}
            rules={[{ required: true }]}
          >
            <Input
              value={data.api.upload.controller.image_url_key}
              disabled={disabled}
              onChange={(e) => {
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    upload: {
                      ...data.api.upload,
                      controller: {
                        ...data.api.upload.controller,
                        image_url_key: e.target.value,
                      },
                    },
                  },
                })
              }}
            />
          </Form.Item>

          <Form.Item
            label="删除 id 键"
            name={['data', 'api', 'upload', 'controller', 'deleted_id_key']}
            rules={[{ required: true }]}
          >
            <Input
              value={data.api.upload.controller.deleted_id_key}
              disabled={disabled}
              onChange={(e) => {
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    upload: {
                      ...data.api.upload,
                      controller: {
                        ...data.api.upload.controller,
                        deleted_id_key: e.target.value,
                      },
                    },
                  },
                })
              }}
            />
          </Form.Item>

          <Form.Item label="图片缓存键">
            <Input
              value={data.api.upload.controller.thumb_key}
              disabled={disabled}
              onChange={(e) => {
                handleChange({
                  ...data,
                  api: {
                    ...data.api,
                    upload: {
                      ...data.api.upload,
                      controller: {
                        ...data.api.upload.controller,
                        thumb_key: e.target.value,
                      },
                    },
                  },
                })
              }}
            />
          </Form.Item>
        </Space>
      </Form.Item>
    </>
  )
}

export default ApiSetting
