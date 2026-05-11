import { invoke } from '@tauri-apps/api/core';

export type Lang = 'en' | 'zh';

class I18nStore {
  currentLang = $state<Lang>('zh');
}

export const i18n = new I18nStore();

const dict: Record<Lang, Record<string, string>> = {
  en: {
    'settings.title': 'Settings',
    'settings.back': '← Pets',
    'settings.display': '⚙ Display',
    'display.title': 'Display',
    'display.petsDir': 'Pets Directory',
    'display.scale': 'Pet Scale',
    'display.alwaysOnTop': 'Always on Top',
    'display.autostart': 'Launch at Startup',
    'display.language': 'Language',
    'home.title': 'Pets',
    'home.new': '+ New',
    'home.import': 'Import',
    'home.newPet': 'New Pet',
    'home.name': 'Name',
    'home.frameSize': 'Frame size',
    'home.displayScale': 'Display scale',
    'home.create': 'Create',
    'home.cancel': 'Cancel',
    'home.noPets': 'No pets. Create one or import from folder.',
    'home.active': 'active',
    'detail.back': '← Back to pets',
    'detail.interactions': 'Interactions',
    'detail.animations': 'Animations',
    'detail.config': 'Config',
    'config.title': 'Config',
    'config.name': 'Name',
    'config.frameWidth': 'Frame Width',
    'config.frameHeight': 'Frame Height',
    'config.displayScale': 'Display Scale',
    'config.defaultState': 'Default State',
    'config.save': 'Save Changes',
    'state.title': 'Interactions',
    'state.add': '+ Add State',
    'state.entry': 'Entry Animation',
    'state.transitions': 'Transitions',
    'state.addTransition': '+ Add Transition',
    'state.noTransitions': 'No transitions.',
    'state.event': 'Event',
    'state.target': 'Target State',
    'state.override': 'Override Animation',
    'state.useTarget': 'Use target entry',
    'animation.title': 'Animations',
    'animation.add': '+ Add',
    'animation.import': 'Import Image',
    'animation.name': 'Name',
    'animation.source': 'Source',
    'animation.frameTime': 'Frame (ms)',
    'animation.frames': 'Frames',
    'animation.perRow': 'Per Row',
    'animation.loop': 'Loop',
    'animation.duration': 'Duration',
    'animation.empty': 'No animations. Click "+ Add" to create one.',
    'animation.save': 'Save Changes',
    'animation.assets': 'Image Assets',
    'animation.noImages': 'No image files.',
    'animation.file': 'File',
    'animation.usage': 'Usage',
    'animation.unused': 'Unused',
    'animation.deleteFile': 'Delete file',
  },
  zh: {
    'settings.title': '设置',
    'settings.back': '← 宠物列表',
    'settings.display': '⚙ 显示设置',
    'display.title': '显示',
    'display.petsDir': '宠物目录',
    'display.scale': '宠物缩放',
    'display.alwaysOnTop': '置顶',
    'display.autostart': '开机自启动',
    'display.language': '语言',
    'home.title': '宠物',
    'home.new': '+ 新建',
    'home.import': '导入',
    'home.newPet': '新建宠物',
    'home.name': '名称',
    'home.frameSize': '帧大小',
    'home.displayScale': '显示缩放',
    'home.create': '创建',
    'home.cancel': '取消',
    'home.noPets': '没有宠物。创建一个或从文件夹导入。',
    'home.active': '当前',
    'detail.back': '← 返回宠物列表',
    'detail.interactions': '交互',
    'detail.animations': '动画',
    'detail.config': '配置',
    'config.title': '配置',
    'config.name': '名称',
    'config.frameWidth': '帧宽',
    'config.frameHeight': '帧高',
    'config.displayScale': '显示缩放',
    'config.defaultState': '默认状态',
    'config.save': '保存更改',
    'state.title': '交互',
    'state.add': '+ 添加状态',
    'state.entry': '入口动画',
    'state.transitions': '状态转换',
    'state.addTransition': '+ 添加转换',
    'state.noTransitions': '没有状态转换。',
    'state.event': '事件',
    'state.target': '目标状态',
    'state.override': '覆盖动画',
    'state.useTarget': '使用目标入口',
    'animation.title': '动画',
    'animation.add': '+ 添加',
    'animation.import': '导入图片',
    'animation.name': '名称',
    'animation.source': '素材',
    'animation.frameTime': '帧时间 (ms)',
    'animation.frames': '帧数',
    'animation.perRow': '每行',
    'animation.loop': '循环',
    'animation.duration': '持续时间',
    'animation.empty': '没有动画。点击 "+ 添加" 创建一个。',
    'animation.save': '保存更改',
    'animation.assets': '图片素材',
    'animation.noImages': '没有图片文件。',
    'animation.file': '文件',
    'animation.usage': '使用情况',
    'animation.unused': '未使用',
    'animation.deleteFile': '删除文件',
  },
};

export function t(key: string): string {
  const _ = i18n.currentLang;
  return dict[i18n.currentLang]?.[key] ?? dict['en']?.[key] ?? key;
}

export async function loadLanguage() {
  try {
    const lang = await invoke<string>('get_language');
    if (lang === 'en' || lang === 'zh') {
      i18n.currentLang = lang;
    }
  } catch (e) {
    console.error('loadLanguage:', e);
  }
}

export async function setLanguage(lang: Lang) {
  i18n.currentLang = lang;
  try {
    await invoke('set_language', { lang });
  } catch (e) {
    console.error('setLanguage:', e);
  }
}
