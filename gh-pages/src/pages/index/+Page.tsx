// SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
// SPDX-License-Identifier: CC-BY-SA-4.0 AND GPL-3.0-or-later

import { InstallTabs } from "../../components/InstallTabs.tsx";
import { Usage } from "../../content/release.tsx";
import styles from "./Page.module.css";

export function Page() {
  return (
    <>
      <header class={styles.header}>
        <img
          class={styles.logo}
          src="/gdvm-godot.svg"
          alt="An illustration of the gdvm logo unpacking the Godot logo out of a box"
          width="400"
          height="311"
        />
        <br />
        <img
          class={styles.logo}
          src="/gdvm-textmark.svg"
          alt="gdvm textmark"
          width="200"
          height="75"
        />
        <p class={styles.tagline}>
          Easily install, manage, and switch between Godot versions.
        </p>
        <div class={styles.actions}>
          <a
            class={styles.btn}
            href="https://github.com/adalinesimonian/gdvm"
            rel="noopener"
          >
            <img alt="GitHub logo" src="/github-logo.svg" /> View on GitHub
          </a>
          <a
            class={styles.btn}
            href="https://bsky.app/profile/gdvm.io"
            rel="noopener"
          >
            <img alt="Bluesky logo" src="/bluesky-logo.svg" />
            Follow on Bluesky
          </a>
        </div>
      </header>

      <div class={styles.wave}>
        <svg viewBox="0 0 1440 320" preserveAspectRatio="none">
          <path
            d="M0,128L48,122.7C96,117,192,107,288,122.7C384,139,480,181,576,176C672,171,768,117,864,96C960,75,1056,85,1152,112C1248,139,1344,181,1392,202.7L1440,224V320H0Z"
            fill="var(--bg)"
          />
        </svg>
      </div>

      <main class={styles.main}>
        <section>
          <p class={styles.getStarted}>
            To get started, run the following command in your terminal:
          </p>
          <InstallTabs />
        </section>

        <h2>What is gdvm?</h2>

        <section>
          <p>
            Godot Version Manager (gdvm) is a tool designed to simplify the
            installation, management, and switching between different versions
            of the Godot Engine.
          </p>
          <p>
            Whether you're working on multiple projects or need to test features
            across various Godot versions, you'll never need to manually fuss
            with Godot installations again.
          </p>
          <p class={styles.unaffiliated}>
            gdvm is a community-driven project, not affiliated with Godot Engine
            or the Godot Foundation.
          </p>
        </section>

        <section>
          <h3>Supported Platforms</h3>
          <ul>
            <li>Windows (64-bit, 32-bit, and 64-bit ARM)</li>
            <li>macOS (64-bit Intel and Apple Silicon)</li>
            <li>Linux (64-bit, 32-bit, and 64-bit ARM)</li>
          </ul>
        </section>

        <section>
          <Usage />
        </section>
      </main>

      <footer class={styles.footer}>
        <p>&copy; 2024 Adaline Simonian.</p>
        <p>
          Page content is licensed under{" "}
          <a
            href="https://creativecommons.org/licenses/by-sa/4.0/"
            rel="license noopener"
          >
            CC BY-SA 4.0
          </a>
          .
          <br />
          Page source code is licensed under{" "}
          <a
            href="https://www.gnu.org/licenses/gpl-3.0.html"
            rel="license noopener"
          >
            GPL-3.0-or-later
          </a>
          .
        </p>
      </footer>
    </>
  );
}
