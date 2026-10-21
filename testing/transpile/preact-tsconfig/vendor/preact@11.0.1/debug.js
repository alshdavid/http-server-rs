import{options as i,Fragment as W,Component as U}from"./index.js";import"./devtools.js";var z={};function re(){z={}}function w(t){return t.type===W?"Fragment":typeof t.type=="function"?t.type.displayName||t.type.name:typeof t.type=="string"?t.type:"#text"}var M=[],S=[],F=new WeakMap,P=new WeakMap;function L(){return M.length>0?M[M.length-1]:null}var R=!0;function l(t){for(var a=[t],s=t;(s=F.get(s))!=null;)a.push(s);return a.reduce(function(v,d){v+="  in "+w(d);var p=d.__source;return p?v+=" (at "+p.fileName+":"+p.lineNumber+")":R&&console.warn("Add @babel/plugin-transform-react-jsx-source to get a more detailed component stack. Note that you should not add it to production builds of your App for bundle size reasons."),R=!1,v+`
`},"")}function V(){var t=i.__b,a=i.diffed,s=i.vnode,v=i.__r,d=i.__e,p=function(n,f,j){M.length=Math.min(M.length,n[0]+f),S.length=Math.min(S.length,n[1]+j)};i.diffed=function(n){var f=P.get(n);f&&(P.delete(n),p(f,0,0)),a&&a(n)},i.__b=function(n){typeof n.type=="function"&&(P.set(n,[M.length,S.length]),n.type!=W&&M.push(n)),t&&t(n)},i.__e=function(n,f,j,I){var g=P.get(f);g&&p(g,+(M[g[0]]==f),+(S[g[1]]==f));try{d(n,f,j,I)}catch(C){throw g&&(P.delete(f),p(g,0,0)),C}},i.vnode=function(n){F.set(n,S.length>0?S[S.length-1]:null),s&&s(n)},i.__r=function(n){(function(f){return typeof f.type=="function"&&f.type!=W})(n)&&S.push(n),v&&v(n)}}function ne(){var t=L();return t?l(t):null}var G=["ref"],K=typeof WeakMap=="function";function A(t){var a=[];return t.__k&&t.__k.forEach(function(s){if(s&&typeof s.type=="function"){if(s.props.__P)return;a.push.apply(a,A(s))}else s&&typeof s.type=="string"&&a.push(s.type)}),a}function J(t){return t?typeof t.type=="function"?t.props.__P?t.props.__P.localName||"":t.__==null?t.__e!=null&&t.__e.parentNode!=null?t.__e.parentNode.localName:"":J(t.__):t.type:""}var Q=U.prototype.setState;function D(t){return t==="table"||t==="tfoot"||t==="tbody"||t==="thead"||t==="td"||t==="tr"||t==="th"}U.prototype.setState=function(t,a){return this.__v==null&&this.state==null&&console.warn(`Calling "this.setState" inside the constructor of a component is a no-op and might be a bug in your application. Instead, set "this.state = {}" directly.

`+l(L())),Q.call(this,t,a)};var Z=/^(address|article|aside|blockquote|details|div|dl|fieldset|figcaption|figure|footer|form|h1|h2|h3|h4|h5|h6|header|hgroup|hr|main|menu|nav|ol|p|pre|search|section|table|ul)$/,ee=U.prototype.forceUpdate;function b(t){var a=t.props,s=w(t),v="";for(var d in a)if(a.hasOwnProperty(d)&&d!=="children"){var p=a[d];typeof p=="function"&&(p="function "+(p.displayName||p.name)+"() {}"),p=Object(p)!==p||p.toString?p+"":Object.prototype.toString.call(p),v+=" "+d+"="+JSON.stringify(p)}var n=a.children;return"<"+s+v+(n&&n.length?">..</"+s+">":" />")}U.prototype.forceUpdate=function(t){return this.__v==null?console.warn(`Calling "this.forceUpdate" inside the constructor of a component is a no-op and might be a bug in your application.

`+l(L())):this.__P==null&&console.warn(`Can't call "this.forceUpdate" on an unmounted component. This is a no-op, but it indicates a memory leak in your application. To fix, cancel all subscriptions and asynchronous tasks in the componentWillUnmount method.

`+l(this.__v)),ee.call(this,t)},i.__m=function(t,a){var s=t.type,v=a.map(function(d){return d&&d.localName}).filter(Boolean);console.error('Expected a DOM node of type "'+s+'" but found "'+v.join(", ")+`" as available DOM-node(s), this is caused by the SSR'd HTML containing different DOM-nodes compared to the hydrated one.

`+l(t))},(function(){V();var t,a=!1,s=i.__b,v=i.diffed,d=i.vnode,p=i.__r,n=i.__e,f=i.__,j=i.__h,I=K?{useEffect:new WeakMap,useLayoutEffect:new WeakMap,lazyPropTypes:new WeakMap}:null,g=[];i.__e=function(e,r,o,h){var c=t&&t.error===e&&t;c&&(c.done=!0);try{if(r&&r.__c&&typeof e.then=="function"){var m=e;e=new Error("Missing Suspense. The throwing component was: "+w(r));for(var _=r;_;_=_.__)if(_.__c&&_.__c.__c){e=m;break}if(e instanceof Error)throw e}(h=h||{}).componentStack=l(r),n(e,r,o,h)}catch(E){var k=t={error:E,vnode:c&&E===c.error?c.vnode:r};throw queueMicrotask(function(){if(!k.done){var O=(function(y){for(var u=y;u&&(typeof u.type!="function"||u.type==W);)u=u.__;return u||y})(k.vnode);console.error("An error occurred in the <"+w(O)+`> component, and no error boundary handled it.

`+l(O))}t==k&&(t=void 0)}),E}typeof e.then!="function"&&console.error(e)},i.__=function(e,r){if(!r)throw new Error(`Undefined parent passed to render(), this is the second argument.
Check if the element is available in the DOM/has the correct id.`);var o;switch(r.nodeType){case 1:case 11:case 9:o=!0;break;default:o=!1}if(!o){var h=w(e);throw new Error("Expected a valid HTML node as a second argument to render.	Received "+r+" instead: render(<"+h+" />, "+r+");")}f&&f(e,r)},i.__b=function(e){t=void 0;var r,o,h,c,m=e.type;if(a=!0,m===void 0)throw new Error(`Undefined component passed to createElement()

You likely forgot to export your component or might have mixed up default and named imports`+b(e)+`

`+l(e));if(m!=null&&typeof m=="object")throw m.__k!==void 0&&m.__e!==void 0?new Error("Invalid type passed to createElement(): "+m+`

Did you accidentally pass a JSX literal as JSX twice?

  let My`+w(e)+" = "+b(m)+`;
  let vnode = <My`+w(e)+` />;

This usually happens when you export a JSX literal and not the component.

`+l(e)):new Error("Invalid type passed to createElement(): "+(Array.isArray(m)?"array":m));if(e.ref!==void 0&&typeof e.ref!="function"&&typeof e.ref!="object"&&!("$$typeof"in e))throw new Error(`Component's "ref" property should be a function, or an object created by createRef(), but got [`+typeof e.ref+`] instead
`+b(e)+`

`+l(e));if(typeof e.type=="string"){for(var _ in e.props)if(_[0]==="o"&&_[1]==="n"&&typeof e.props[_]!="function"&&e.props[_]!=null)throw new Error(`Component's "`+_+'" property should be a function, but got ['+typeof e.props[_]+`] instead
`+b(e)+`

`+l(e))}if(typeof e.type=="function"&&e.type.propTypes){if(e.type.displayName==="Lazy"&&I&&!I.lazyPropTypes.has(e.type)){var k="PropTypes are not supported on lazy(). Use propTypes on the wrapped component itself. ";try{var E=e.type();I.lazyPropTypes.set(e.type,!0),console.warn(k+"Component wrapped in lazy() is "+w(E))}catch{console.warn(k+"We will log the wrapped component's name once it is loaded.")}}var O=(function(y,u){if(y==null)return{};var T={};for(var N in y)if({}.hasOwnProperty.call(y,N)){if(u.indexOf(N)!==-1)continue;T[N]=y[N]}return T})(e.props,G);r=e.type.propTypes,o=O,h=w(e),c=function(){return l(e)},Object.keys(r).forEach(function(y){var u;try{u=r[y](o,y,h,"prop",null,"SECRET_DO_NOT_PASS_THIS_OR_YOU_WILL_BE_FIRED")}catch(T){u=T}u&&!(u.message in z)&&(z[u.message]=!0,console.error("Failed prop type: "+u.message+`
`+c()))})}s&&s(e)};var C,H=0;i.__r=function(e){p&&p(e),a=!0;var r=e.__c;if(r===C?H++:H=1,H>=25)throw new Error("Too many re-renders. This is limited to prevent an infinite loop which may lock up your browser. The component causing this is: "+w(e));C=r},i.__h=function(e,r,o){if(!e||!a)throw new Error("Hook can only be invoked from render methods.");j&&j(e,r,o)};var Y=function(e,r){return{get:function(){var o="get"+e+r;g&&g.indexOf(o)<0&&(g.push(o),console.warn("getting vnode."+e+" is deprecated, "+r))},set:function(){var o="set"+e+r;g&&g.indexOf(o)<0&&(g.push(o),console.warn("setting vnode."+e+" is not allowed, "+r))}}},X={nodeName:Y("nodeName","use vnode.type"),attributes:Y("attributes","use vnode.props"),children:Y("children","use vnode.props.children")},$=Object.create({},X);i.vnode=function(e){var r=e.props;if(e.type!==null&&r!=null&&("__source"in r||"__self"in r)){var o=e.props={};for(var h in r){var c=r[h];h==="__source"?e.__source=c:h==="__self"?e.__self=c:o[h]=c}}e.__proto__=$,d&&d(e)},i.diffed=function(e){var r,o=e.type,h=e.__;if(e.__k&&e.__k.forEach(function(x){if(typeof x=="object"&&x&&x.type===void 0){var B=Object.keys(x).join(",");throw new Error("Objects are not valid as a child. Encountered an object with the keys {"+B+`}.

`+l(e))}}),e.__c===C&&(H=0),typeof o=="string"&&(D(o)||o==="p"||o==="a"||o==="button")){var c=J(h);if(c!==""&&D(o))o==="table"&&c!=="td"&&D(c)?console.error("Improper nesting of table. Your <table> should not have a table-node parent."+b(e)+`

`+l(e)):o!=="thead"&&o!=="tfoot"&&o!=="tbody"||c==="table"?o==="tr"&&c!=="thead"&&c!=="tfoot"&&c!=="tbody"?console.error("Improper nesting of table. Your <tr> should have a <thead/tbody/tfoot> parent."+b(e)+`

`+l(e)):o==="td"&&c!=="tr"?console.error("Improper nesting of table. Your <td> should have a <tr> parent."+b(e)+`

`+l(e)):o==="th"&&c!=="tr"&&console.error("Improper nesting of table. Your <th> should have a <tr>."+b(e)+`

`+l(e)):console.error("Improper nesting of table. Your <thead/tbody/tfoot> should have a <table> parent."+b(e)+`

`+l(e));else if(o==="p"){var m=A(e).filter(function(x){return Z.test(x)});m.length&&console.error("Improper nesting of paragraph. Your <p> should not have "+m.join(", ")+" as child-elements."+b(e)+`

`+l(e))}else o!=="a"&&o!=="button"||A(e).indexOf(o)!==-1&&console.error("Improper nesting of interactive content. Your <"+o+"> should not have other "+(o==="a"?"anchor":"button")+" tags as child-elements."+b(e)+`

`+l(e))}if(a=!1,v&&v(e),e.__k!=null)for(var _=[],k=0;k<e.__k.length;k++){var E=e.__k[k];if(E&&E.key!=null){var O=E.key;if(_.indexOf(O)!==-1){console.error('Following component has two or more children with the same key attribute: "'+O+`". This may cause glitches and misbehavior in rendering process. Component: 

`+b(e)+`

`+l(e));break}_.push(O)}}if(e.__c!=null&&e.__c.__H!=null){var y=e.__c.__H.__;if(y)for(var u=0;u<y.length;u+=1){var T=y[u];if(T.__H){for(var N=0;N<T.__H.length;N++)if((r=T.__H[N])!=r){var q=w(e);console.warn("Invalid argument passed to hook. Hooks should not be called with NaN in the dependency array. Hook index "+u+" in component "+q+" was called with NaN.")}}}}}})();export{ne as captureOwnerStack,L as getCurrentVNode,w as getDisplayName,l as getOwnerStack,re as resetPropWarnings,V as setupComponentStack};
