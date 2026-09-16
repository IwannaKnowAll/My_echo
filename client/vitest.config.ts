import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

// 独立测试配置：不修改 vite.config.ts，仅用于 vitest。
export default defineConfig({
  plugins: [vue()],
  test: {
    environment: 'jsdom',
  },
});