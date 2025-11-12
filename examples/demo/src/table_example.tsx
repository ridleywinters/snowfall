import React, { JSX } from "react";
import { css, Div, useCSSLocal } from "@raiment-ui";
import * as core from "@raiment-core";

type DashboardEntry = {
    category: string;
    name: string;
    status: number;
    notes: string;
};

class Database {
    entries: DashboardEntry[] = [];

    add(category: string, name: string, status: number): void {
        this.entries.push({ category, name, status, notes: "" });
    }
}

export function TableExample(): JSX.Element {
    const database = React.useMemo(() => {
        const db = new Database();
        db.add("System", "Authentication Service", 7);
        db.add("System", "Data Processing Service", 5);
        db.add("Network", "API Gateway", 4);
        db.add("Network", "Load Balancer", 8);
        db.add("Database", "User Database", 6);
        db.add("Database", "Analytics Database", 3);
        return db;
    }, []);

    const cl = useCSSLocal(
        css`
            .table {
                display: inline-grid;
                grid-template-columns: auto auto auto auto auto;
                border: 1px solid #ddd;
                border-radius: 4px;
                overflow: hidden;
            }

            .header, .row {
                display: contents; /* so child cells share the same grid columns */
            }
            .header .cell {
                font-weight: bold;
                color: #eee;
                background-color: #333;
            }
            .cell {
                padding: 2px;
                border-bottom: 1px solid #ddd;
                margin: 0;

                &.category {
                    padding-left: 1em;
                    color: #555;
                    text-transform: lowercase;
                }
                &.name {
                    padding-right: 1em;
                }
                &.status-value {
                    text-align: center;
                    color: #ccc;
                    font-size: 11px;
                    padding: 0 2em;
                }
            }
        `,
    );

    return (
        <Div sl="m32" cl={cl}>
            <h1>Table Example</h1>

            <Div cl="table">
                <Div cl="header">
                    <Div cl="cell"></Div>
                    <Div cl="cell">name</Div>
                    <Div cl="cell"></Div>
                    <Div cl="cell">status</Div>
                    <Div cl="cell"></Div>
                </Div>
                {database.entries.map((entry, index) => (
                    <DashboardRowView key={index} entry={entry} />
                ))}
            </Div>
        </Div>
    );
}

function DashboardRowView({ entry }: { entry: DashboardEntry }): JSX.Element {
    const width = core.clampi(entry.status * 10, 0, 100);
    const color = entry.status <= 3
        ? "#E53935"
        : entry.status < 5
        ? "#FB8C00"
        : entry.status < 7
        ? "#FDD835"
        : entry.status <= 10
        ? "#43A047"
        : "#FF00FF";

    return (
        <Div cl="row" sl="flex-row-center">
            <Div cl="cell category" sl="font-size-11 width-8em">{entry.category}</Div>
            <Div cl="cell name">{entry.name}</Div>
            <Div cl="cell status-value">{entry.status}</Div>
            <Div sl="cell flex-row-center">
                <Div sl="relative width-100px bg-#ddd height-20px">
                    <Div
                        sl={[
                            "absolute left-0 top-0 height-20px",
                            `width-${width}px bg-${color}`,
                        ]}
                    />
                </Div>
            </Div>
            <Div cl="cell">{entry.notes}</Div>
        </Div>
    );
}
