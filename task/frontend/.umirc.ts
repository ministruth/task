import { defineConfig } from '@umijs/max';
import { PLUGIN_ID } from './src/config';
const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');

export default defineConfig({
  publicPath:
    process.env.NODE_ENV === 'production'
      ? `/_plugin/${PLUGIN_ID}/`
      : undefined,
  // mfsu bug
  mfsu: false,
  // symlink fix
  chainWebpack(memo, _) {
    memo.resolve.symlinks(false);
    memo
      .plugin('monaco-editor-webpack-plugin')
      .use(MonacoWebpackPlugin, [{ languages: ['rust'] }]);
  },
  access: {
    strictMode: true,
  },
  qiankun: {
    slave: {},
  },
  exportStatic: {},
  // qiankun dependency
  model: {},
  hash: true,
  antd: {},
  locale: {
    default: 'en-US',
    antd: true,
    title: true,
    baseNavigator: true,
  },
  request: {
    dataField: 'data',
  },
  fastRefresh: true,
  proxy: {
    '/api/': {
      ws: true,
      target: 'http://localhost:8080/',
      changeOrigin: true,
    },
  },
});
