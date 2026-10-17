import { applyBootAppearance } from './lib/bootAppearance'

if (import.meta.env.DEV) {
  document.title = '(DEV) Atrium'
}

// Mark the platform as soon as the bundled entry starts evaluating, before styles and React,
// so platform-specific styles (such as the macOS traffic-light inset) apply on the first paint.
document.documentElement.dataset.platform = /Macintosh|Mac OS X/i.test(navigator.userAgent)
  ? 'macos'
  : 'other'

// Paint the startup screen with the last known theme; App.tsx takes over after hydration.
applyBootAppearance()
