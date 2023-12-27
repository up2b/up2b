import React, { useState } from 'react'
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

interface Data {
  token: string
  config: ApiConfig
}

interface ApiSettingProps {
  code: ManagerCode
  token?: string
  config?: ApiConfig
  onChange: (data: Data) => void
}

const ALLOWED_FORMATS = ['PNG', 'JPEG', 'GIF', 'WEBP', 'BMP']

const initApiConfig: ApiConfig = {
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
  const [data, setData] = useState<Data>({
    token: token ?? '',
    config: config ?? initApiConfig,
  })

  const filteredFormats = ALLOWED_FORMATS.filter(
    (o) => !config?.upload.allowed_formats.includes(o),
  )

  const disabled = code === 'SMMS'

  const handleChange = (data: Data) => {
    setData(data)

    onChange(data)
  }

  return (
    <>
      <Form.Item label="TOKEN">
        <Input.Password
          placeholder="输入 token"
          value={data.token || ''}
          onChange={(e) => handleChange({ ...data, token: e.target.value })}
        />
      </Form.Item>

      <Divider>图片列表</Divider>

      <Form.Item label="接口">
        <Input
          placeholder="输入图片列表接口"
          defaultValue={data.config.list.url}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              config: {
                ...data.config,
                list: { ...data.config.list, url: e.target.value },
              },
            })
          }
        />
      </Form.Item>

      <Form.Item label="请求方法">
        <Select
          defaultValue={data.config.list.method.type}
          disabled={disabled}
          onChange={(value) =>
            handleChange({
              ...data,
              config: {
                ...data.config,
                list: {
                  ...data.config.list,
                  method: { ...data.config.list.method, type: value },
                },
              },
            })
          }
        >
          <Select.Option value="GET">GET</Select.Option>
          <Select.Option value="POST">POST</Select.Option>
        </Select>
      </Form.Item>

      <Form.Item>
        <Space wrap>
          <Form.Item label="图片数组键">
            <Input
              placeholder="输入图片数组键名"
              defaultValue={data.config.list.controller.items_key}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  config: {
                    ...data.config,
                    list: {
                      ...data.config.list,
                      controller: {
                        ...data.config.list.controller,
                        items_key: e.target.value,
                      },
                    },
                  },
                })
              }
            />
          </Form.Item>

          <Form.Item label="图片地址键">
            <Input
              placeholder="输入图片地址键名"
              defaultValue={data.config.list.controller.image_url_key}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  config: {
                    ...data.config,
                    list: {
                      ...data.config.list,
                      controller: {
                        ...data.config.list.controller,
                        image_url_key: e.target.value,
                      },
                    },
                  },
                })
              }
            />
          </Form.Item>

          <Form.Item label="图片删除 id 键">
            <Input
              placeholder="输入图片删除 id 键名"
              defaultValue={data.config.list.controller.deleted_id_key}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  config: {
                    ...data.config,
                    list: {
                      ...data.config.list,
                      controller: {
                        ...data.config.list.controller,
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
              defaultValue={data.config.list.controller.thumb_key}
              disabled={disabled}
              onChange={(e) =>
                handleChange({
                  ...data,
                  config: {
                    ...data.config,
                    list: {
                      ...data.config.list,
                      controller: {
                        ...data.config.list.controller,
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

      <Form.Item label="接口">
        <Input
          placeholder="输入图片删除接口"
          defaultValue={data.config.delete.url}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              config: {
                ...data.config,
                delete: {
                  ...data.config.delete,
                  url: e.target.value,
                },
              },
            })
          }
        />
      </Form.Item>

      <Form.Item label="请求方法">
        <Select
          defaultValue={data.config.delete.method.type}
          disabled={disabled}
          onChange={(e) =>
            handleChange({
              ...data,
              config: {
                ...data.config,
                delete: {
                  ...data.config.delete,
                  method:
                    e === 'GET'
                      ? {
                          ...(data.config.delete.method as ApiDeleteGetMethod),
                          type: 'GET',
                        }
                      : { type: 'POST' },
                },
              },
            })
          }
        >
          <Select.Option value="GET">GET</Select.Option>
          <Select.Option value="POST">POST</Select.Option>
        </Select>
      </Form.Item>

      <Form.Item
        label="是否有响应体"
        tooltip="删除图片如果有响应体则应该传入响应体中的有效属性键，否则以响应状态码判断是否成功"
      >
        <Switch
          defaultValue={data.config.delete.controller.type === 'JSON'}
          disabled={disabled}
          onChange={(v) =>
            handleChange({
              ...data,
              config: {
                ...data.config,
                delete: {
                  ...data.config.delete,
                  controller: v
                    ? {
                        ...(data.config.delete
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

      {data.config.delete.controller.type === 'JSON' ? (
        <Form.Item>
          <Space>
            <Form.Item label="成功键">
              <Input
                placeholder="删除成功与否的键名"
                defaultValue={data.config.delete.controller.key}
                disabled={disabled}
                onChange={(e) =>
                  handleChange({
                    ...data,
                    config: {
                      ...data.config,
                      delete: {
                        ...data.config.delete,
                        controller: {
                          ...(data.config.delete
                            .controller as ApiDeleteJsonController),
                          key: e.target.value,
                        },
                      },
                    },
                  })
                }
              />
            </Form.Item>

            <Form.Item label="成功的值" tooltip="删除成功时值应该是什么">
              <Input
                addonBefore={
                  <Select
                    defaultValue={
                      typeof data.config.delete.controller.should_be
                    }
                    disabled={disabled}
                  >
                    <Select.Option value="boolean">布尔</Select.Option>
                    <Select.Option value="number">数字</Select.Option>
                    <Select.Option value="string">字符串</Select.Option>
                  </Select>
                }
                placeholder="删除成功的值"
                defaultValue={data.config.delete.controller.should_be}
                disabled={disabled}
                onChange={(e) =>
                  handleChange({
                    ...data,
                    config: {
                      ...data.config,
                      delete: {
                        ...data.config.delete,
                        controller: {
                          ...(data.config.delete
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

            <Form.Item label="失败消息键">
              <Input
                placeholder="删除失败的消息键名"
                defaultValue={data.config.delete.controller.message_key}
                disabled={disabled}
                onChange={(e) =>
                  handleChange({
                    ...data,
                    config: {
                      ...data.config,
                      delete: {
                        ...data.config.delete,
                        controller: {
                          ...(data.config.delete
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
        <Form.Item label="接口">
          <Input
            placeholder="输入上传图片接口"
            defaultValue={data.config.upload.url}
            disabled={disabled}
            onChange={(e) =>
              handleChange({
                ...data,
                config: {
                  ...data.config,
                  upload: {
                    ...data.config.upload,
                    url: e.target.value,
                  },
                },
              })
            }
          />
        </Form.Item>

        <Form.Item label="最大体积">
          <InputNumber
            placeholder="输入允许的最大体积"
            defaultValue={
              data.config.upload.max_size
                ? data.config.upload.max_size / 1024 / 1024
                : undefined
            }
            disabled={disabled}
            addonAfter="MB"
            onChange={(v) =>
              v &&
              handleChange({
                ...data,
                config: {
                  ...data.config,
                  upload: {
                    ...data.config.upload,
                    max_size: v,
                  },
                },
              })
            }
          />
        </Form.Item>

        <Form.Item label="允许的格式">
          <Select
            mode="multiple"
            placeholder="选择图片格式"
            value={config?.upload.allowed_formats ?? []}
            onChange={(values) => {
              handleChange({
                ...data,
                config: {
                  ...data.config,
                  upload: {
                    ...data.config.upload,
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
            defaultValue={data.config.upload.content_type.type}
            disabled={disabled}
            onChange={(e) => {
              handleChange({
                ...data,
                config: {
                  ...data.config,
                  upload: {
                    ...data.config.upload,
                    content_type:
                      e.target.value === 'JSON'
                        ? {
                            ...(data.config.upload
                              .content_type as ApiUploadJsonContentType),
                            type: 'JSON',
                          }
                        : {
                            ...(data.config.upload
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

        {data.config.upload.content_type.type === 'MULTIPART' ? (
          <Space>
            <Form.Item
              label="上传类型"
              tooltip="流式响应支持上传进度，流式上传失败时可尝试更换为 bytes"
            >
              <Radio.Group
                value={data.config.upload.content_type.file_kind ?? 'STREAM'}
                disabled={disabled}
                onChange={(e) => {
                  handleChange({
                    ...data,
                    config: {
                      ...data.config,
                      upload: {
                        ...data.config.upload,
                        content_type: {
                          ...(data.config.upload
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

            <Form.Item label="图片的表单键">
              <Input
                defaultValue={data.config.upload.content_type.file_part_name}
                disabled={disabled}
                onChange={(e) => {
                  handleChange({
                    ...data,
                    config: {
                      ...data.config,
                      upload: {
                        ...data.config.upload,
                        content_type: {
                          ...(data.config.upload
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
            <Form.Item label="图片数组的表单键">
              <Input defaultValue={data.config.upload.content_type.key} />
            </Form.Item>

            <Form.Item label="除图片之外的其他 json 数据">
              <Input.TextArea />
            </Form.Item>
          </>
        )}

        <Space>
          <Form.Item label="图片键">
            <Input
              defaultValue={data.config.upload.controller.image_url_key}
              disabled={disabled}
              onChange={(e) => {
                handleChange({
                  ...data,
                  config: {
                    ...data.config,
                    upload: {
                      ...data.config.upload,
                      controller: {
                        ...data.config.upload.controller,
                        image_url_key: e.target.value,
                      },
                    },
                  },
                })
              }}
            />
          </Form.Item>

          <Form.Item label="删除 id 键">
            <Input
              defaultValue={data.config.upload.controller.deleted_id_key}
              disabled={disabled}
              onChange={(e) => {
                handleChange({
                  ...data,
                  config: {
                    ...data.config,
                    upload: {
                      ...data.config.upload,
                      controller: {
                        ...data.config.upload.controller,
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
              defaultValue={data.config.upload.controller.thumb_key}
              disabled={disabled}
              onChange={(e) => {
                handleChange({
                  ...data,
                  config: {
                    ...data.config,
                    upload: {
                      ...data.config.upload,
                      controller: {
                        ...data.config.upload.controller,
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
