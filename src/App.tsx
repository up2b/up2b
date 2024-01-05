import React, { lazy, useEffect, useState } from 'react'
import { appWindow } from '@tauri-apps/api/window'
import { Tabs, Tooltip, message } from 'antd'
import {
  CloudUploadOutlined,
  UnorderedListOutlined,
  SettingOutlined,
} from '@ant-design/icons'
import { suspense } from '~/advance/index'
import { LazyList, LazyUpload, LazySetting } from '~/lazy'
import { getConfig } from '~/lib'
import '~/App.scss'

const TitleBar =
  import.meta.env.TAURI_PLATFORM === 'windows'
    ? lazy(() => import('~/components/title-bar'))
    : null

const keys: { [K in TabKey]: string } = {
  list: '图片列表',
  settings: '设置',
  upload: '上传',
}

const Home = () => {
  const [messageApi, contextHolder] = message.useMessage()

  const [config, setConfig] = useState<Config | null>(null)

  const [activeKey, setActiveKey] = useState<TabKey>('upload')

  useEffect(() => {
    if (config) return

    getConfig().then((c) => {
      if (!c) {
        setActiveKey('settings')
        messageApi.warning('配置文件不存在，请先选择并配置一个图床')
      } else {
        setConfig(c)
      }
    })
  }, [])

  useEffect(() => {
    if (!activeKey) return

    appWindow.setTitle(keys[activeKey])
  }, [activeKey])

  const disabled =
    !config ||
    !config.auth_config ||
    !config.auth_config[config.using] ||
    (config.auth_config[config.using]!.type === 'API'
      ? !(config!.auth_config[config!.using]! as ApiAuthConfig).token
      : (config.auth_config[config!.using]! as GitAuthConfig).type === 'GIT'
        ? !(config.auth_config[config.using] as GitAuthConfig).token ||
        !(config.auth_config[config.using] as GitAuthConfig).username ||
        !(config.auth_config[config.using] as GitAuthConfig).repository
        : !(config.auth_config[config.using]! as CheveretoAuthConfig)
          .username ||
        !(config.auth_config[config.using]! as CheveretoAuthConfig).password)

  const tabs = [
    {
      label: (
        <Tooltip title="上传" placement="right">
          <CloudUploadOutlined />
        </Tooltip>
      ),
      key: 'upload',
      disabled,
      children: suspense(<LazyUpload />),
      destroyInactiveTabPane: true,
    },
    {
      label: (
        <Tooltip title="图片列表" placement="right">
          <UnorderedListOutlined />
        </Tooltip>
      ),
      key: 'list',
      disabled,
      children: suspense(<LazyList />),
      destroyInactiveTabPane: true,
    },
    {
      label: (
        <Tooltip title="设置" placement="right">
          <SettingOutlined />
        </Tooltip>
      ),
      key: 'settings',
      children: suspense(<LazySetting config={config} setConfig={setConfig} />),
    },
  ]

  return (
    <>
      <div
        data-tauri-drag-region
        style={{
          height: 28,
          position: 'fixed',
          width: '100%',
          zIndex: 9999999,
        }}
      >
        {TitleBar && suspense(<TitleBar title={keys[activeKey]} />)}
      </div>
      <Tabs
        // type="card" // antd 没有对垂直卡片进行适配
        className="tabs"
        tabPosition="left"
        activeKey={activeKey}
        size="large"
        indicatorSize={0}
        centered
        onChange={(k) => setActiveKey(k as TabKey)}
        items={tabs}
      />

      {contextHolder}
    </>
  )
}

export default Home
