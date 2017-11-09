use std::collections::{HashMap, HashSet};
use std::iter::Peekable;
use std::slice::Iter;

use syn::{self, DelimToken, Ident, IntTy, Lit, Path, Token, TokenTree};

use error::*;

use {App, Idle, Init, Resources, Static, Statics, Task, Tasks};

/// Parses the contents of `app! { $App }`
pub fn app(input: &str) -> Result<App> {
    let tts = syn::parse_token_trees(input)?;

    let mut device = None;
    let mut idle = None;
    let mut init = None;
    let mut resources = None;
    let mut root = None;
    let mut tasks = None;

    fields(&tts, |key, tts| {
        match key.as_ref() {
            "device" => {
                ensure!(device.is_none(), "duplicated `device` field");

                device =
                    Some(::parse::path(tts).chain_err(|| "parsing `device`")?);
            }
            "idle" => {
                ensure!(idle.is_none(), "duplicated `idle` field");

                idle = Some(::parse::idle(tts).chain_err(|| "parsing `idle`")?);
            }
            "init" => {
                ensure!(init.is_none(), "duplicated `init` field");

                init = Some(::parse::init(tts).chain_err(|| "parsing `init`")?);
            }
            "resources" => {
                ensure!(resources.is_none(), "duplicated `resources` field");

                resources = Some(
                    ::parse::statics(tts).chain_err(|| "parsing `resources`")?,
                );
            }
            "root" => {
                ensure!(root.is_none(), "duplicated `root` field");

                root = Some(
                    ::parse::path(tts).chain_err(|| "parsing `root`")?,
                );
            }
            "tasks" => {
                ensure!(tasks.is_none(), "duplicated `tasks` field");

                tasks =
                    Some(::parse::tasks(tts).chain_err(|| "parsing `tasks`")?);
            }
            _ => bail!("unknown field: `{}`", key),
        }

        Ok(())
    })?;

    Ok(App {
        _extensible: (),
        device: device.ok_or("`device` field is missing")?,
        idle,
        init,
        resources,
        root,
        tasks,
    })
}

/// Parses a boolean
fn bool(tt: Option<&TokenTree>) -> Result<bool> {
    if let Some(&TokenTree::Token(Token::Literal(Lit::Bool(bool)))) = tt {
        Ok(bool)
    } else {
        bail!("expected boolean, found {:?}", tt);
    }
}

/// Parses a delimited token tree
fn delimited<R, F>(
    tts: &mut Peekable<Iter<TokenTree>>,
    delimiter: DelimToken,
    f: F,
) -> Result<R>
where
    F: FnOnce(&[TokenTree]) -> Result<R>,
{
    let tt = tts.next();
    if let Some(&TokenTree::Delimited(ref block)) = tt {
        ensure!(
            block.delim == delimiter,
            "expected {:?}, found {:?}",
            delimiter,
            block.delim
        );

        f(&block.tts)
    } else {
        bail!("expected a Delimited sequence, found {:?}", tt);
    }
}

/// Parses `$($Ident: $($tt)*,)*`
fn fields<F>(tts: &[TokenTree], mut f: F) -> Result<()>
where
    F: FnMut(&Ident, &mut Peekable<Iter<TokenTree>>) -> Result<()>,
{
    let mut tts = tts.iter().peekable();

    while let Some(tt) = tts.next() {
        let ident = if let TokenTree::Token(Token::Ident(ref id)) = *tt {
            id
        } else {
            bail!("expected Ident, found {:?}", tt);
        };

        let tt = tts.next();
        if let Some(&TokenTree::Token(Token::Colon)) = tt {
        } else {
            bail!("expected Colon, found {:?}", tt);
        }

        f(ident, &mut tts)?;

        let tt = tts.next();
        match tt {
            None | Some(&TokenTree::Token(Token::Comma)) => {}
            _ => bail!("expected Comma, found {:?}", tt),
        }
    }

    Ok(())
}

/// Parses the LHS of `idle: { $Idle }`
fn idle(tts: &mut Peekable<Iter<TokenTree>>) -> Result<Idle> {
    ::parse::delimited(tts, DelimToken::Brace, |tts| {
        let mut path = None;
        let mut resources = None;

        ::parse::fields(tts, |key, tts| {
            match key.as_ref() {
                "path" => {
                    ensure!(path.is_none(), "duplicated `path` field");

                    path = Some(::parse::path(tts)?);
                }
                "resources" => {
                    ensure!(
                        resources.is_none(),
                        "duplicated `resources` field"
                    );

                    resources = Some(::parse::resources(tts)
                        .chain_err(|| "parsing `resources`")?);
                }
                _ => bail!("unknown field: `{}`", key),
            }

            Ok(())
        })?;

        Ok(Idle {
            _extensible: (),
            path,
            resources,
        })
    })
}

/// Parses the LHS of `init: { $Init }`
fn init(tts: &mut Peekable<Iter<TokenTree>>) -> Result<Init> {
    ::parse::delimited(tts, DelimToken::Brace, |tts| {
        let mut path = None;

        ::parse::fields(tts, |key, tts| {
            match key.as_ref() {
                "path" => {
                    ensure!(path.is_none(), "duplicated `path` field");

                    path = Some(::parse::path(tts)?);
                }
                _ => bail!("unknown field: `{}`", key),
            }

            Ok(())
        })?;

        Ok(Init {
            _extensible: (),
            path,
        })
    })
}

/// Parses `[$($Ident,)*]`
fn resources(tts: &mut Peekable<Iter<TokenTree>>) -> Result<Resources> {
    ::parse::delimited(tts, DelimToken::Bracket, |tts| {
        let mut idents = HashSet::new();

        let mut tts = tts.iter().peekable();
        while let Some(tt) = tts.next() {
            if let &TokenTree::Token(Token::Ident(ref ident)) = tt {
                ensure!(
                    !idents.contains(ident),
                    "ident {} listed more than once"
                );

                idents.insert(ident.clone());

                if let Some(tt) = tts.next() {
                    ensure!(
                        tt == &TokenTree::Token(Token::Comma),
                        "expected Comma, found {:?}",
                        tt
                    );

                    if tts.peek().is_none() {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                bail!("expected Ident, found {:?}", tt);
            }
        }

        Ok(idents)
    })
}

/// Parses `$Ty = $Expr`
fn static_(tts: &mut Iter<TokenTree>) -> Result<Static> {
    let mut fragments = vec![];
    loop {
        if let Some(tt) = tts.next() {
            match *tt {
                TokenTree::Token(Token::Eq) => break,
                TokenTree::Token(Token::Semi) => {
                    let ty = syn::parse_type(&format!("{}", quote!(#(#fragments)*)))?;
                    return Ok(Static {
                        _extensible: (),
                        expr: None,
                        ty,
                    });
                }
                _ => fragments.push(tt),
            }
        } else {
            bail!("expected `=` or `;`, found end of macro");
        }
    }

    let ty = syn::parse_type(&format!("{}", quote!(#(#fragments)*)))?;

    let mut fragments = vec![];
    loop {
        if let Some(tt) = tts.next() {
            if tt == &TokenTree::Token(Token::Semi) {
                break;
            } else {
                fragments.push(tt);
            }
        } else {
            bail!("expected Semicolon, found end of macro");
        }
    }

    ensure!(!fragments.is_empty(), "initial value is missing");
    let expr = quote!(#(#fragments)*);

    Ok(Static {
        _extensible: (),
        expr: Some(expr),
        ty,
    })
}

/// Parses `$($Ident: $Ty = $Expr;)*`
fn statics(tts: &mut Peekable<Iter<TokenTree>>) -> Result<Statics> {
    ::parse::delimited(tts, DelimToken::Brace, |tts| {
        let mut statics = HashMap::new();

        let mut tts = tts.iter();
        while let Some(tt) = tts.next() {
            match tt {
                &TokenTree::Token(Token::Ident(ref id))
                    if id.as_ref() == "static" => {}
                _ => {
                    bail!("expected keyword `static`, found {:?}", tt);
                }
            }

            let tt = tts.next();
            let ident =
                if let Some(&TokenTree::Token(Token::Ident(ref id))) = tt {
                    id
                } else {
                    bail!("expected Ident, found {:?}", tt);
                };

            ensure!(
                !statics.contains_key(ident),
                "resource {} listed more than once",
                ident
            );

            let tt = tts.next();
            if let Some(&TokenTree::Token(Token::Colon)) = tt {
            } else {
                bail!("expected Colon, found {:?}", tt);
            }

            statics.insert(
                ident.clone(),
                ::parse::static_(&mut tts)
                    .chain_err(|| format!("parsing `{}`", ident))?,
            );
        }

        Ok(statics)
    })
}

/// Parses a `Path` from `$($tt)*`
fn path(tts: &mut Peekable<Iter<TokenTree>>) -> Result<Path> {
    let mut fragments = vec![];

    loop {
        if let Some(tt) = tts.peek() {
            if tt == &&TokenTree::Token(Token::Comma) {
                break;
            } else {
                fragments.push(tt.clone());
            }
        } else {
            bail!("expected Comma, found end of macro")
        }

        tts.next();
    }

    Ok(syn::parse_path(&format!("{}", quote!(#(#fragments)*)))?)
}

/// Parses the LHS of `$Ident: { .. }`
fn task(tts: &mut Peekable<Iter<TokenTree>>) -> Result<Task> {
    ::parse::delimited(tts, DelimToken::Brace, |tts| {
        let mut enabled = None;
        let mut path = None;
        let mut priority = None;
        let mut resources = None;

        ::parse::fields(tts, |key, tts| {
            match key.as_ref() {
                "enabled" => {
                    ensure!(enabled.is_none(), "duplicated `enabled` field");

                    enabled = Some(::parse::bool(tts.next())
                        .chain_err(|| "parsing `enabled`")?);
                }
                "path" => {
                    ensure!(path.is_none(), "duplicated `path` field");

                    path = Some(
                        ::parse::path(tts).chain_err(|| "parsing `path`")?,
                    );
                }
                "priority" => {
                    ensure!(priority.is_none(), "duplicated `priority` field");

                    priority = Some(::parse::u8(tts.next())
                        .chain_err(|| "parsing `priority`")?);
                }
                "resources" => {
                    ensure!(
                        resources.is_none(),
                        "duplicated `resources` field"
                    );

                    resources = Some(::parse::resources(tts)
                        .chain_err(|| "parsing `resources`")?);
                }
                _ => bail!("unknown field: `{}`", key),
            }

            Ok(())
        })?;

        Ok(Task {
            _extensible: (),
            enabled,
            path,
            priority,
            resources,
        })
    })
}

/// Parses `$($Ident: { $Task })*`
fn tasks(tts: &mut Peekable<Iter<TokenTree>>) -> Result<Tasks> {
    ::parse::delimited(tts, DelimToken::Brace, |tts| {
        let mut tasks = HashMap::new();

        ::parse::fields(tts, |key, tts| {
            ensure!(
                !tasks.contains_key(key),
                "task {} listed more than once",
                key
            );

            tasks.insert(
                key.clone(),
                ::parse::task(tts)
                    .chain_err(|| format!("parsing task `{}`", key))?,
            );

            Ok(())
        })?;

        Ok(tasks)
    })
}

/// Parses an integer with type `u8`
fn u8(tt: Option<&TokenTree>) -> Result<u8> {
    if let Some(
        &TokenTree::Token(
            Token::Literal(Lit::Int(priority, IntTy::Unsuffixed)),
        ),
    ) = tt
    {
        ensure!(priority < 256, "{} is out of the `u8` range", priority);

        Ok(priority as u8)
    } else {
        bail!("expected integer, found {:?}", tt);
    }
}
