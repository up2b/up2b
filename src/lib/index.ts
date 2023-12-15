export const areObjectsEqual = (obj1: any, obj2: any): boolean => {
  if (typeof obj1 !== 'object' || typeof obj2 !== 'object' || !obj1 || !obj2) {
    return obj1 === obj2
  }

  const keys1 = Object.keys(obj1)
  const keys2 = Object.keys(obj2)

  if (keys1.length !== keys2.length) {
    return false
  }

  for (const key of keys1) {
    if (!keys2.includes(key) || !areObjectsEqual(obj1[key], obj2[key])) {
      return false
    }
  }

  return true
}

export * from './api'
export * from '~/lib/store'
