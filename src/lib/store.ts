const STORAGE_KEY_PREFIX = 'images'

const imageBedKey = (imageBed: ManagerCode) => {
  return `${STORAGE_KEY_PREFIX}.${imageBed}`
}

export const getImagesInStorage = (
  imageBed: ManagerCode = 'SMMS',
): ImageResponseItem[] | undefined => {
  const cached = localStorage.getItem(imageBedKey(imageBed))

  if (!cached) return

  return JSON.parse(cached)
}

export const clearStorage = (imageBed: ManagerCode = 'SMMS') => {
  localStorage.removeItem(imageBedKey(imageBed))
}

export const addImagesInStorage = (
  imageBed: ManagerCode = 'SMMS',
  ...items: ImageResponseItem[]
) => {
  const currentItems = getImagesInStorage(imageBed) ?? []

  currentItems.push(...items)

  setStorage(imageBed, currentItems)
}

const containsImage = (
  target: ImageResponseItem,
  items: ImageResponseItem[],
): boolean => {
  return items.some((item) => item.url === target.url)
}

export const deleteImagesInStorage = (
  imageBed: ManagerCode = 'SMMS',
  ...items: ImageResponseItem[]
) => {
  const currentItems = getImagesInStorage(imageBed) ?? []

  const newItems = currentItems.filter((v) => !containsImage(v, items))

  setStorage(imageBed, newItems)
}

export const setStorage = (
  imageBed: ManagerCode = 'SMMS',
  items: ImageResponseItem[],
) => {
  if (!items) return

  localStorage.setItem(imageBedKey(imageBed), JSON.stringify(items))
}
