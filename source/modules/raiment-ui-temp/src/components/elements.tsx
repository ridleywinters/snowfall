import React, { ElementType, JSX } from "react";
import { StyleLanguage, useStyleLanguage } from "../hooks/use_style_language.tsx";

type TagProps<T extends ElementType> =
    & {
        tag: T;
        sl?: StyleLanguage;
        cl?: string;
    }
    & { [key in `data-${string}`]?: string | number | boolean }
    & React.ComponentPropsWithoutRef<T>;

function Element<T extends ElementType>({
    tag,
    sl,
    cl,
    className,
    children,
    ...props
}: TagProps<T>): JSX.Element {
    const Component = tag;
    const slClassName = useStyleLanguage(sl);
    const computedClass = [className, cl, slClassName].filter((c) => c).join(" ") || undefined;

    return (
        <Component
            data-component={props["data-component"]}
            className={computedClass}
            {...props as any}
        >
            {children}
        </Component>
    );
}

function createExtendedElement<T extends ElementType>(tag: T) {
    return function WrappedElement(props: Omit<TagProps<T>, "tag">): JSX.Element {
        return <Element tag={tag} {...props as any} />;
    };
}

export const Div = createExtendedElement("div");
export const Span = createExtendedElement("span");
export const Anchor = createExtendedElement("a");
