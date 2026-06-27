// SPDX-FileCopyrightText: Copyright (C) 2025 Adaline Simonian
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of gdvm.
//
// gdvm is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

export interface V2ReleaseAsset {
  sha512: string;
  urls: string[];
}

export interface V2ReleaseVariant {
  [platform: string]: V2ReleaseAsset;
}

export interface V2ReleaseVariants {
  [variant: string]: V2ReleaseVariant;
}

export interface V2Release {
  schema: 2;
  /** ISO 8601 timestamp. */
  updated_at: string;
  version: string;
  variants: V2ReleaseVariants;
}

export interface V2ReleaseIndexEntry {
  version: string;
  variants: Record<string, string[]>;
  path: string;
}

export interface V2ReleaseIndex {
  schema: 2;
  releases: V2ReleaseIndexEntry[];
}

export interface V2RegistryManifest {
  schema: 2;
  name: "gdvm-official";
  description: string;
  /** ISO 8601 timestamp. */
  updated_at: string;
}
