import React, { useState } from 'react'
import { appWindow } from '@tauri-apps/api/window'
import './index.scss'

const Close = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 16 16"
    xmlns="http://www.w3.org/2000/svg"
    fill="currentColor"
  >
    <path
      fill-rule="evenodd"
      clip-rule="evenodd"
      d="M7.116 8l-4.558 4.558.884.884L8 8.884l4.558 4.558.884-.884L8.884 8l4.558-4.558-.884-.884L8 7.116 3.442 2.558l-.884.884L7.116 8z"
    />
  </svg>
)

const Restore = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 16 16"
    xmlns="http://www.w3.org/2000/svg"
    fill="currentColor"
  >
    <path d="M3 5v9h9V5H3zm8 8H4V6h7v7z" />
    <path
      fill-rule="evenodd"
      clip-rule="evenodd"
      d="M5 5h1V4h7v7h-1v1h2V3H5v2z"
    />
  </svg>
)

const Minimize = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 16 16"
    xmlns="http://www.w3.org/2000/svg"
    fill="currentColor"
  >
    <path d="M14 8v1H3V8h11z" />
  </svg>
)

const Maximize = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 16 16"
    xmlns="http://www.w3.org/2000/svg"
    fill="currentColor"
  >
    <path d="M3 3v10h10V3H3zm9 9H4V4h8v8z" />
  </svg>
)

interface ButtonProps {
  children: React.ReactNode
  isClose?: boolean
  onClick: () => void
}

const Button = ({ children, isClose, onClick }: ButtonProps) => {
  return (
    <div className={'button' + (isClose ? ' close' : '')} onClick={onClick}>
      {children}
    </div>
  )
}

interface TitleBarProps {
  title: string
}

const TitleBar = ({ title }: TitleBarProps) => {
  const [isMaximized, setIsMaximized] = useState(false)

  const toggleIsMaximized = async () => {
    await appWindow.toggleMaximize()
    setIsMaximized((pre) => !pre)
  }

  return (
    <div id="title-bar" data-tauri-drag-region>
      {title}

      <div id="buttons">
        <Button onClick={() => appWindow.minimize()}>
          <Minimize />
        </Button>

        {!isMaximized ? (
          <Button onClick={toggleIsMaximized}>
            <Maximize />
          </Button>
        ) : (
          <Button onClick={toggleIsMaximized}>
            <Restore />
          </Button>
        )}

        <Button isClose onClick={() => appWindow.close()}>
          <Close />
        </Button>
      </div>
    </div>
  )
}

export default TitleBar
