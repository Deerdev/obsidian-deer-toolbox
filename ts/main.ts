import {
  App,
  Editor,
  MarkdownView,
  Modal,
  Notice,
  Plugin,
  PluginSettingTab,
  Setting,
  TFile,
} from "obsidian";
import DeerBasePlugin from "./base_plugin";
import { process_web_image_from_content } from "../pkg/obsidian_deer_toolbox";

interface MyPluginSettings {
  mediaRootPath: string;
}

const DEFAULT_SETTINGS: MyPluginSettings = {
  mediaRootPath: "media",
};

export default class DeerToolboxPlugin extends DeerBasePlugin {
  settings: MyPluginSettings;

  downloadWebImages = async () => {
    await process_web_image_from_content(this);
  };

  async onload() {
    await this.loadSettings();

    this.addSettingTab(new DeerToolboxSettingTab(this.app, this));

    this.addCommand({
      id: "deer-toolbox-download-web-images",
      name: "Download web images",
      callback: this.downloadWebImages,
    });
  }

  onunload() {}

  async loadSettings() {
    this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
  }

  async saveSettings() {
    try {
      await this.saveData(this.settings);
    } catch (error) {
      this.displayError(error);
    }
  }

  get mediaPath(): string {
    return this.settings.mediaRootPath;
  }

  // backup web images
  // async downloadWebImages(file: TFile) {
  // }
}

class SampleModal extends Modal {
  constructor(app: App) {
    super(app);
  }

  onOpen() {
    const { contentEl } = this;
    contentEl.setText("Woah!");
  }

  onClose() {
    const { contentEl } = this;
    contentEl.empty();
  }
}

class DeerToolboxSettingTab extends PluginSettingTab {
  plugin: DeerToolboxPlugin;

  constructor(app: App, plugin: DeerToolboxPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display(): void {
    const { containerEl } = this;

    containerEl.empty();

    containerEl.createEl("h2", { text: "Deer Toolbox" });

    new Setting(containerEl)
      .setName("Media folder")
      .setDesc("Folder to keep all downloaded media files.")
      .addText((text) =>
        text
          .setValue(this.plugin.settings.mediaRootPath)
          .onChange(async (value) => {
            this.plugin.settings.mediaRootPath = value;
            await this.plugin.saveSettings();
          })
      );
  }
}
