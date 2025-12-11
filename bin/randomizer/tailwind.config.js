import { join } from 'path';
import { skeleton, contentPath } from '@skeletonlabs/skeleton/plugin';
import * as themes from '@skeletonlabs/skeleton/themes';
import forms from '@tailwindcss/forms';
import typography from '@tailwindcss/typography';

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: [
    './src/**/*.{html,js,svelte,ts}',
    contentPath(import.meta.url, 'svelte')
  ],
  theme: {
    extend: {},
  },
  plugins: [
    forms,
    typography,
    skeleton({
      themes: [themes.cerberus, themes.rose]
    })
  ]
};
