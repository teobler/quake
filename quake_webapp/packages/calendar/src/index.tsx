import React from 'react';
import {render, unmountComponentAtNode} from 'react-dom';
import QuakeCalendar from './QuakeCalendar';

// from samples: https://github.com/sarvabowmen/rating-webcomponent/blob/master/src/ReactElement.js
class ReactElement extends HTMLElement {
  static observedAttributes = ['title', 'content'];
  private _innerHTML: string | undefined;
  private observer: any;

  connectedCallback() {
    this._innerHTML = this.innerHTML;
    this.mount();
  }

  disconnectedCallback() {
    this.unmount();
    this.observer.disconnect();
  }

  attributeChangedCallback() {
    this.unmount();
    this.mount();
  }

  mount() {
    const props = {
      ...this.getProps(this.attributes),
      ...this.getEvents()
    };

    // @ts-ignore
    render(<QuakeCalendar {...props}/>, this);
  }

  unmount() {
    unmountComponentAtNode(this);
  }

  getProps(attributes: any) {
    return [...attributes]
      .filter(attr => attr.name !== 'style')
      .map(attr => this.convert(attr.name, attr.value))
      .reduce((props, prop) =>
        ({...props, [prop.name]: prop.value}), {});
  }

  getEvents() {
    return Object.values(this.attributes)
      .filter(key => /on([a-z].*)/.exec(key.name))
      .reduce((events, ev) => ({
        ...events,
        [ev.name]: (args: any) =>
          this.dispatchEvent(new CustomEvent(ev.name, {...args}))
      }), {});
  }

  convert(attrName: any, attrValue: any) {
    let value = attrValue;
    if (attrValue === 'true' || attrValue === 'false')
      value = attrValue === 'true';
    else if (!isNaN(attrValue) && attrValue !== '')
      value = +attrValue;
    else if ((/^{.*}/.exec(attrValue)) || (/^\[.*]/.exec(attrValue)) )
      value = JSON.parse(attrValue);
    return {
      name: attrName,
      value: value
    };
  }

}

customElements.define('quake-calendar', ReactElement);
