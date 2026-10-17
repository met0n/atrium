import { getCurrentWindow } from '@tauri-apps/api/window'
import { useEffect, useState } from 'react'

import { isMacOS } from '../lib/platform'

export type NativeWindowControls = {
  /** The OS draws the window controls (macOS traffic lights), so the title bar must not. */
  native: boolean
  /** The native controls are on screen over the title bar and need room; false in full screen. */
  inset: boolean
}

/** macOS keeps its traffic lights, overlaid on the title bar by `tauri.macos.conf.json`. */
export function useNativeWindowControls(): NativeWindowControls {
  const native = isMacOS()
  const [fullscreen, setFullscreen] = useState(false)

  useEffect(() => {
    if (!native) return
    const win = getCurrentWindow()
    let cancelled = false
    let unlisten: (() => void) | null = null
    const refresh = () => {
      void win
        .isFullscreen()
        .then((value) => {
          if (!cancelled) setFullscreen(value)
        })
        .catch(() => {})
    }

    refresh()
    void win
      .onResized(refresh)
      .then((stopListening) => {
        if (cancelled) stopListening()
        else unlisten = stopListening
      })
      .catch(() => {})

    return () => {
      cancelled = true
      unlisten?.()
    }
  }, [native])

  return { native, inset: native && !fullscreen }
}
