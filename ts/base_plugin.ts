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

export default class DeerBasePlugin extends Plugin {
  file: TFile;

  getActiveFile(): TFile | null {
    return this.app.workspace.getActiveFile();
  }

  async getActiveFileContent(): Promise<string> {
    const file = this.getActiveFile();
    if (!file) {
      this.displayError("No active file");
      return "";
    }
    this.file = file;
    return await this.app.vault.cachedRead(file);
  }

  async saveFileContent(content: string) {
    if (!this.file) {
      return
    }
    await this.app.vault.modify(this.file, content);
  }

  async saveBinaryResource(fileName: string, fileData: ArrayBuffer) {
    await app.vault.createBinary(fileName, fileData);
  }

  displayError(error: Error | string, file?: TFile): void {
    if (file) {
      new Notice(
        `DeerToolbox: Error while handling file ${
          file.name
        }, ${error.toString()}`
      );
    } else {
      new Notice(`DeerToolbox: ${error.toString()}`);
    }

    console.error(`DeerToolbox: ${error}`);
  }
}
