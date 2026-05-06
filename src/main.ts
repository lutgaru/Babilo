/*
 * Babilo - Copyright (C) 2026 Lutgaru
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import './styles.css';
import './tailwind.css';
import { loadTailwindSheet } from './lib/tailwind-styles';
import './components/bbl-shell';

loadTailwindSheet().catch(err => console.error('❌ Error loading Tailwind sheet:', err));