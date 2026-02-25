/**
 * Arx Pi Extension — decision journaling
 *
 * Tools:
 *   journal — record decisions, observations, and notes via `wobot journal`
 *
 * Usage:
 *   pi -e /path/to/arx/pi-extension
 */

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { StringEnum } from "@mariozechner/pi-ai";
import { Type } from "@sinclair/typebox";

export default function (pi: ExtensionAPI) {
	pi.registerTool({
		name: "journal",
		label: "Journal",
		description:
			"Record decisions, observations, and notes in the arx journal. Use after brainstorming, plan approval, or architectural choices.",
		parameters: Type.Object({
			type: StringEnum(["decision", "observation", "note"] as const, {
				description: "Entry type",
			}),
			title: Type.String({ description: "Brief title for the entry" }),
			scope: Type.Optional(
				Type.Array(Type.String(), {
					description: "Scope tags, e.g. ['architecture', 'phase-d']",
				}),
			),
		}),

		async execute(_toolCallId, params, signal, _onUpdate, _ctx) {
			const args = ["journal", "record", "--type", params.type, "--title", params.title];
			if (params.scope) {
				for (const s of params.scope) {
					args.push("--scope", s);
				}
			}

			const result = await pi.exec("wobot", args, { signal, timeout: 10000 });

			if (result.code !== 0) {
				return {
					content: [{ type: "text", text: `journal error (exit ${result.code}):\n${result.stderr}` }],
					details: { exitCode: result.code },
					isError: true,
				};
			}

			return {
				content: [{ type: "text", text: result.stdout || "Journal entry recorded." }],
				details: { exitCode: result.code },
			};
		},
	});
}
