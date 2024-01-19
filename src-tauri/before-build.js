import fs from 'fs'
import path from 'path'
import { Readable } from 'stream'
import { finished } from 'stream/promises'

const downloadNasm = async () => {
  const resp = await fetch(
    'https://www.nasm.us/pub/nasm/releasebuilds/2.16.01/win64/nasm-2.16.01-win64.zip',
  )

  if (!fs.existsSync('bin')) await mkdir('bin')
  const destination = path.resolve('./bin', 'nasm.zip')
  const fileStream = fs.createWriteStream(destination, { flags: 'wx' })
  await finished(Readable.fromWeb(resp.body).pipe(fileStream))
}

const updateVersion = async () => {
  const infoBuffer = fs.readFileSync('../package.json')
  const info = JSON.parse(infoBuffer.toString())

  const { version } = info

  if (process.platform !== 'win32') {
    return version
  }

  if (version.indexOf('-') === -1) {
    return
  }

  const newVersion = version.split('-')[0]
  info.version = newVersion

  fs.writeFileSync('../package.json', JSON.stringify(info))

  await downloadNasm()

  console.log(version)
}

updateVersion()
