use std::fmt;
pub use types::{Qualified, TypeVariable, Type, Constraint};
pub use types::Type::{Application, Constructor, Variable};
pub use module::{TypeDeclaration, DataDefinition, Newtype};
pub use module::LiteralData::{Integral, Fractional, String, Char};
use module;
use interner::*;
pub use renamer::Name;

pub struct Module<Ident> {
    pub classes: Vec<Class<Ident>>,
    pub data_definitions: Vec<DataDefinition<Name>>,
    pub newtypes: Vec<Newtype<Name>>,
    pub instances: Vec<Instance<Ident>>,
    pub bindings: Vec<Binding<Ident>>
}

impl Module<Id> {
    pub fn from_expr(expr: Expr<Id>) -> Module<Id> {
        Module {
            classes: vec![],
            data_definitions: vec![],
            newtypes: Vec::new(),
            instances: vec![],
            bindings: vec![Binding {
                name: Id::new(Name { name: intern("main"), uid: 0 }, expr.get_type().clone(), vec![]),
                expression: expr
            }]
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Class<Ident> {
    pub constraints: Vec<Constraint<Name>>,
    pub name : Name,
    pub variable : TypeVariable,
    pub declarations : Vec<module::TypeDeclaration<Name>>,
    pub bindings: Vec<Binding<Ident>>
}

#[derive(Clone)]
pub struct Instance<Ident = InternedStr> {
    pub bindings : Vec<Binding<Ident>>,
    pub constraints : Vec<Constraint<Name>>,
    pub typ : Type,
    pub classname : Name
}

#[derive(Clone, PartialEq)]
pub struct Binding<Ident> {
    pub name: Ident,
    pub expression: Expr<Ident>
}

#[derive(Clone, PartialEq)]
pub struct Alternative<Ident> {
    pub pattern : Pattern<Ident>,
    pub expression : Expr<Ident>
}

#[derive(Clone, PartialEq)]
pub enum Pattern<Ident> {
    Constructor(Ident, Vec<Ident>),
    Identifier(Ident),
    Number(int),
    WildCard
}

#[derive(Clone, PartialEq)]
pub struct LiteralData {
    pub typ: Type,
    pub value: Literal_
}

pub type Literal_ = module::LiteralData;

#[derive(Clone, PartialEq)]
pub enum Expr<Ident> {
    Identifier(Ident),
    Apply(Box<Expr<Ident>>, Box<Expr<Ident>>),
    Literal(LiteralData),
    Lambda(Ident, Box<Expr<Ident>>),
    Let(Vec<Binding<Ident>>, Box<Expr<Ident>>),
    Case(Box<Expr<Ident>>, Vec<Alternative<Ident>>)
}

impl fmt::Show for LiteralData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl <T: fmt::Show> fmt::Show for Binding<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} = {:?}", self.name, self.expression)
    }
}

impl <T: fmt::Show> fmt::Show for Expr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Expr::*;
        write_core_expr!(*self, f,)
    }
}
impl <T: fmt::Show> fmt::Show for Alternative<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.pattern, self.expression)
    }
}
impl <T: fmt::Show> fmt::Show for Pattern<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Pattern::Identifier(ref s) => write!(f, "{:?}", s),
            Pattern::Number(ref i) => write!(f, "{:?}", i),
            Pattern::Constructor(ref name, ref patterns) => {
                try!(write!(f, "({:?} ", name));
                for p in patterns.iter() {
                    try!(write!(f, " {:?}", p));
                }
                write!(f, ")")
            }
            Pattern::WildCard => write!(f, "_")
        }
    }
}

///Trait which provides access to the Type of any struct which implements it.
pub trait Typed {
    fn get_type<'a>(&'a self) -> &'a Type;
}

impl <Ident: Typed> Typed for Expr<Ident> {
    fn get_type<'a>(&'a self) -> &'a Type {
        match self {
            &Expr::Identifier(ref i) => i.get_type(),
            &Expr::Literal(ref lit) => &lit.typ,
            &Expr::Apply(ref func, _) => {
                match func.get_type() {
                    &Type::Application(_, ref a) => { let a2: &Type = *a; a2 }
                    x => panic!("The function in Apply must be a type application, found {:?}", x)
                }
            }
            &Expr::Lambda(ref arg, _) => arg.get_type(),
            &Expr::Let(_, ref body) => body.get_type(),
            &Expr::Case(_, ref alts) => alts[0].expression.get_type()
        }
    }
}
impl <Ident: Typed> Typed for Pattern<Ident> {
    fn get_type<'a>(&'a self) -> &'a Type {
        match *self {
            Pattern::Identifier(ref name) => name.get_type(),
            Pattern::Constructor(ref name, _) => name.get_type(),
            Pattern::Number(_) => panic!(),
            Pattern::WildCard(..) => panic!()
        }
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, o: &str) -> bool {
        self.name == intern(o)
    }
}

///Id is a Name combined with a type
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Id<T = Name> {
    pub name: T,
    pub typ: Qualified<Type, Name>
}

impl fmt::Show for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.name)
    }
}

impl <T> Id<T> {
    pub fn new(name: T, typ: Type, constraints: Vec<Constraint<Name>>) -> Id<T> {
        Id { name: name, typ: module::qualified(constraints, typ) }
    }
}

impl Str for Id {
    fn as_slice<'a>(&'a self) -> &'a str {
        self.name.name.as_slice()
    }
}

impl <T> Typed for Id<T> {
    fn get_type<'a>(&'a self) -> &'a Type {
        &self.typ.value
    }
}


mod ref_ {
    use super::*;
    use super::Expr::*;
    ///Visitor for the types in the core language.
    ///visit_ is called at the every value in the tree, if it is overriden
    ///the appropriate walk_ methods need to be called to continue walking
    pub trait Visitor<Ident> {
        fn visit_expr(&mut self, expr: &Expr<Ident>) {
            walk_expr(self, expr)
        }
        fn visit_alternative(&mut self, alt: &Alternative<Ident>) {
            walk_alternative(self, alt)
        }
        fn visit_pattern(&mut self, _pattern: &Pattern<Ident>) {
        }
        fn visit_binding(&mut self, binding: &Binding<Ident>) {
            walk_binding(self, binding);
        }
        fn visit_module(&mut self, module: &Module<Ident>) {
            walk_module(self, module);
        }
    }

    pub fn walk_module<Ident>(visitor: &mut Visitor<Ident>, module: &Module<Ident>) {
        for bind in module.bindings.iter() {
            visitor.visit_binding(bind);
        }
    }

    pub fn walk_binding<Ident>(visitor: &mut Visitor<Ident>, binding: &Binding<Ident>) {
        visitor.visit_expr(&binding.expression);
    }

    pub fn walk_expr<Ident>(visitor: &mut Visitor<Ident>, expr: &Expr<Ident>) {
        match expr {
            &Apply(ref func, ref arg) => {
                visitor.visit_expr(*func);
                visitor.visit_expr(*arg);
            }
            &Lambda(_, ref body) => visitor.visit_expr(*body),
            &Let(ref binds, ref e) => {
                for b in binds.iter() {
                    visitor.visit_binding(b);
                }
                visitor.visit_expr(*e);
            }
            &Case(ref e, ref alts) => {
                visitor.visit_expr(*e);
                for alt in alts.iter() {
                    visitor.visit_alternative(alt);
                }
            }
            _ => ()
        }
    }

    pub fn walk_alternative<Ident>(visitor: &mut Visitor<Ident>, alt: &Alternative<Ident>) {
        visitor.visit_expr(&alt.expression);
    }
}

pub mod mutable {
    use super::*;
    use super::Expr::*;
    
    pub trait Visitor<Ident> {
        fn visit_expr(&mut self, expr: &mut Expr<Ident>) {
            walk_expr(self, expr)
        }
        fn visit_alternative(&mut self, alt: &mut Alternative<Ident>) {
            walk_alternative(self, alt)
        }
        fn visit_pattern(&mut self, _pattern: &mut Pattern<Ident>) {
        }
        fn visit_binding(&mut self, binding: &mut Binding<Ident>) {
            walk_binding(self, binding);
        }
        fn visit_module(&mut self, module: &mut Module<Ident>) {
            walk_module(self, module);
        }
    }

    pub fn walk_module<Ident, V: Visitor<Ident>>(visitor: &mut V, module: &mut Module<Ident>) {
        for bind in module.bindings.iter_mut() {
            visitor.visit_binding(bind);
        }
    }

    pub fn walk_binding<Ident, V: Visitor<Ident>>(visitor: &mut V, binding: &mut Binding<Ident>) {
        visitor.visit_expr(&mut binding.expression);
    }

    pub fn walk_expr<Ident, V: Visitor<Ident>>(visitor: &mut V, expr: &mut Expr<Ident>) {
        match expr {
            &Apply(ref mut func, ref mut arg) => {
                visitor.visit_expr(*func);
                visitor.visit_expr(*arg);
            }
            &Lambda(_, ref mut body) => visitor.visit_expr(*body),
            &Let(ref mut binds, ref mut e) => {
                for b in binds.iter_mut() {
                    visitor.visit_binding(b);
                }
                visitor.visit_expr(*e);
            }
            &Case(ref mut e, ref mut alts) => {
                visitor.visit_expr(*e);
                for alt in alts.iter_mut() {
                    visitor.visit_alternative(alt);
                }
            }
            _ => ()
        }
    }

    pub fn walk_alternative<Ident, V: Visitor<Ident>>(visitor: &mut V, alt: &mut Alternative<Ident>) {
        visitor.visit_expr(&mut alt.expression);
    }
}

pub mod result {
    use super::*;
    use super::Expr::*;

    ///A visitor which takes the structs as values and in turn expects a value in return
    ///so that it can rebuild the tree
    pub trait Visitor<Ident> {
        fn visit_expr(&mut self, expr: Expr<Ident>) -> Expr<Ident> {
            walk_expr(self, expr)
        }
        fn visit_alternative(&mut self, alt: Alternative<Ident>) -> Alternative<Ident> {
            walk_alternative(self, alt)
        }
        fn visit_pattern(&mut self, pattern: Pattern<Ident>) -> Pattern<Ident> {
            pattern
        }
        fn visit_binding(&mut self, binding: Binding<Ident>) -> Binding<Ident> {
            walk_binding(self, binding)
        }
        fn visit_module(&mut self, module: Module<Ident>) -> Module<Ident> {
            walk_module(self, module)
        }
    }

    pub fn walk_module<Ident>(visitor: &mut Visitor<Ident>, mut module: Module<Ident>) -> Module<Ident> {
        let mut bindings = vec![];
        ::std::mem::swap(&mut module.bindings, &mut bindings);
        module.bindings = bindings.into_iter()
            .map(|bind| visitor.visit_binding(bind))
            .collect();
        module
    }

    pub fn walk_binding<Ident>(visitor: &mut Visitor<Ident>, binding: Binding<Ident>) -> Binding<Ident> {
        let Binding { name: name, expression: expression } = binding;
        Binding {
            name: name,
            expression: visitor.visit_expr(expression)
        }
    }

    pub fn walk_expr<Ident>(visitor: &mut Visitor<Ident>, expr: Expr<Ident>) -> Expr<Ident> {
        match expr {
            Apply(func, arg) => {
                let f = visitor.visit_expr(*func);
                let a = visitor.visit_expr(*arg);
                Apply(box f, box a)
            }
            Lambda(x, body) => Lambda(x, box visitor.visit_expr(*body)),
            Let(binds, e) => {
                let bs: Vec<Binding<Ident>> = binds.into_iter().map(|b| {
                    visitor.visit_binding(b)
                }).collect();
                Let(bs, box visitor.visit_expr(*e))
            }
            Case(e, alts) => {
                let e2 = visitor.visit_expr(*e);
                let alts2: Vec<Alternative<Ident>> = alts.into_iter()
                    .map(|alt| visitor.visit_alternative(alt))
                    .collect();
                Case(box e2, alts2)
            }
            expr => expr
        }
    }

    pub fn walk_alternative<Ident>(visitor: &mut Visitor<Ident>, alt: Alternative<Ident>) -> Alternative<Ident> {
        let Alternative { pattern: pattern, expression: expression } = alt;
        Alternative { pattern: visitor.visit_pattern(pattern), expression: visitor.visit_expr(expression) }
    }
}

///The translate module takes the AST and translates it into the simpler core language.
pub mod translate {
    use module;
    use core::*;
    use core::Expr::*;
    use types::{function_type_, list_type, char_type, bool_type};
    use interner::*;
    use renamer::NameSupply;
    use deriving::*;
    use std::collections::HashMap;

    struct Translator<'a> {
        name_supply: NameSupply,
        functions_in_class: &'a mut (FnMut(Name) -> (&'a TypeVariable, &'a [TypeDeclaration<Name>]) + 'a)
    }
    
    #[derive(Show)]
    struct Equation<'a>(&'a [(Id<Name>, Pattern<Id<Name>>)], (&'a [Binding<Id<Name>>], &'a module::Match<Name>));

    pub fn translate_expr(expr: module::TypedExpr<Name>) -> Expr<Id<Name>> {
        let mut translator = Translator { name_supply: NameSupply::new(), functions_in_class: |_| panic!() };
        translator.translate_expr(expr)
    }

    pub fn translate_modules(modules: Vec<module::Module<Name>>) -> Vec<Module<Id<Name>>> {
        let mut map = HashMap::new();
        for class in modules.iter().flat_map(|m| m.classes.iter()) {
            map.insert(class.name.clone(), (class.variable.clone(), class.declarations.clone()));
        }
        let mut translator = Translator {
            name_supply: NameSupply::new(),
            functions_in_class: |name| {
                let &(ref var, ref decls) = map.get(&name);
                (var, decls.as_slice())
            }
        };
        modules.into_iter()
            .map(|module| translate_module_(&mut translator, module))
            .collect()
    }
    pub fn translate_module(module: module::Module<Name>) -> Module<Id<Name>> {
        translate_modules(vec!(module)).pop().unwrap()
    }
    fn translate_module_<'a>(translator: &mut Translator<'a>, module: module::Module<Name>) -> Module<Id<Name>> {
        let module::Module { name : _name,
            imports : _imports,
            bindings : bindings,
            typeDeclarations : _typeDeclarations,
            newtypes : newtypes,
            classes : classes,
            instances : instances,
            dataDefinitions : dataDefinitions,
            fixity_declarations : _fixity_declarations
        } = module;

        let mut new_instances: Vec<Instance<Id<Name>>> = Vec::new();

        let classes2 : Vec<Class<Id>> = classes.into_iter().map(|class| {
            let module::Class {
                constraints: cs,
                name : name,
                variable : var,
                declarations : decls,
                bindings: bindings
            } = class;
            Class {
                constraints: cs,
                name: name,
                variable: var,
                declarations: decls,
                bindings: translator.translate_bindings(bindings)
            }
        }).collect();

        for instance in instances.into_iter() {
            let module::Instance {
                classname: classname,
                typ: instance_type,
                constraints: constraints,
                bindings: bindings
            } = instance;
            let bs: Vec<Binding<Id<Name>>> = translator.translate_bindings(bindings).into_iter().collect();
            new_instances.push(Instance {
                constraints: constraints,
                typ: instance_type,
                classname: classname,
                bindings: bs
            });
        }
        let bs: Vec<Binding<Id<Name>>> = translator.translate_bindings(bindings).into_iter().collect();
        for data in dataDefinitions.iter() {
            generate_deriving(&mut new_instances, data);
        }
        for instance in new_instances.iter_mut() {
            let (class_var, class_decls) = (translator.functions_in_class)(instance.classname);
            let defaults = create_default_stubs(class_var, class_decls, instance);
            let mut temp = Vec::new();
            ::std::mem::swap(&mut temp, &mut instance.bindings);
            let vec: Vec<Binding<Id<Name>>> = temp.into_iter().chain(defaults.into_iter()).collect();
            instance.bindings = vec;
        }
        Module {
            classes: classes2,
            data_definitions: dataDefinitions,
            newtypes: newtypes,
            bindings: bs,
            instances: new_instances
        }
    }

    ///Creates stub functions for each undeclared function in the instance
    fn create_default_stubs(class_var: &TypeVariable, class_decls: &[TypeDeclaration<Name>], instance: &Instance<Id<Name>>) -> Vec<Binding<Id<Name>>> {
        class_decls.iter()
            .filter(|decl| instance.bindings.iter().find(|bind| bind.name.as_slice().ends_with(decl.name.as_slice())).is_none())
            .map(|decl| {
                debug!("Create default function for {:?} ({:?}) {:?}", instance.classname, instance.typ, decl.name);
                //The stub functions will naturally have the same type as the function in the class but with the variable replaced
                //with the instance's type
                let mut typ = decl.typ.clone();
                ::typecheck::replace_var(&mut typ.value, class_var, &instance.typ);
                {
                    let context = ::std::mem::replace(&mut typ.constraints, Vec::new());
                    //Remove all constraints which refer to the class's variable
                    let vec_context: Vec<Constraint<Name>> = context.into_iter()
                        .filter(|c| c.variables[0] != *class_var)
                        .collect();
                    typ.constraints = vec_context;
                }
                let Qualified { value: typ, constraints: constraints } = typ;
                let default_name = module::encode_binding_identifier(instance.classname.name, decl.name.name);
                let typ_name = module::extract_applied_type(&instance.typ).ctor().name;
                let instance_fn_name = module::encode_binding_identifier(typ_name, decl.name.name);

                //Example stub for undeclared (/=)
                //(/=) = #Eq/=
                Binding {
                    name: Id::new(Name { name: instance_fn_name, uid: decl.name.uid }, typ.clone(), constraints.clone()),
                    expression: Identifier(Id::new(Name { name: default_name, uid: decl.name.uid }, typ, constraints))
                }
            })
            .collect()
    }
impl <'a> Translator<'a> {
    fn translate_match(&mut self, matches: module::Match<Name>) -> Expr<Id<Name>> {
        match matches {
            module::Match::Simple(e) => self.translate_expr(e),
            module::Match::Guards(ref gs) => self.translate_guards(unmatched_guard(), *gs)
        }
    }

    pub fn translate_expr(&mut self, input_expr: module::TypedExpr<Name>) -> Expr<Id<Name>> {
        //Checks if the expression is lambda not bound by a let binding
        //if it is then we wrap the lambda in a let binding
        let is_lambda = match &input_expr.expr {
            &module::Expr::Lambda(_, _) => true,
            _ => false
        };
        if is_lambda {
            let module::TypedExpr { typ: typ, expr: expr, ..} = input_expr;
            match expr {
                module::Expr::Lambda(arg, body) => {
                    //TODO need to make unique names for the lambdas created here
                    let argname = match arg {
                        module::Pattern::Identifier(arg) => arg,
                        module::Pattern::WildCard => Name { name: intern("_"), uid: -1 },
                        _ => panic!("Core translation of pattern matches in lambdas are not implemented")
                    };
                    let l = Lambda(Id::new(argname, typ.clone(), vec![]), box self.translate_expr_rest(*body));
                    let id = Id::new(self.name_supply.from_str("#lambda"), typ.clone(), vec![]);
                    let bind = Binding { name: id.clone(), expression: l };
                    Let(vec![bind], box Identifier(id))
                }
                _ => panic!()
            }
        }
        else {
            self.translate_expr_rest(input_expr)
        }
    }

    fn translate_expr_rest(&mut self, input_expr: module::TypedExpr<Name>) -> Expr<Id<Name>> {
        let module::TypedExpr { typ: typ, expr: expr, ..} = input_expr;
        match expr {
            module::Expr::Identifier(s) => Identifier(Id::new(s, typ, vec![])),
            module::Expr::Apply(func, arg) => Apply(box self.translate_expr(*func), box self.translate_expr(*arg)),
            module::Expr::OpApply(lhs, op, rhs) => {
                let l = box self.translate_expr(*lhs);
                let r = box self.translate_expr(*rhs);
                let func_type = function_type_(l.get_type().clone(),
                                function_type_(r.get_type().clone(),
                                               typ));
                Apply(box Apply(box Identifier(Id::new(op, func_type, vec![])), l), r)
            }
            module::Expr::Literal(l) => Literal(Literal { typ: typ, value: l }),
            module::Expr::Lambda(arg, body) => {
                match arg {
                    module::Pattern::Identifier(arg) => Lambda(Id::new(arg, typ, vec![]), box self.translate_expr_rest(*body)),
                    module::Pattern::WildCard => Lambda(Id::new(Name { name: intern("_"), uid: -1 }, typ, vec![]), box self.translate_expr_rest(*body)),
                    _ => panic!("Core translation of pattern matches in lambdas are not implemented")
                }
            }
            module::Expr::Let(bindings, body) => {
                let bs = self.translate_bindings(bindings);
                Let(bs, box self.translate_expr(*body))
            }
            module::Expr::Case(expr, alts) => {
                self.translate_case(*expr, alts)
            }
            module::Expr::IfElse(pred, if_true, if_false) => {
                Case(box self.translate_expr(*pred), vec![
                    Alternative { pattern: bool_pattern("True"), expression: self.translate_expr(*if_true) },
                    Alternative { pattern: bool_pattern("False"), expression: self.translate_expr(*if_false) }
                    ])
            }
            module::Expr::Do(bindings, expr) => {
                let mut result = self.translate_expr(*expr);
                for bind in bindings.into_iter().rev() {
                    result = match bind {
                        module::DoBinding::DoExpr(e) => {
                            let core = self.translate_expr(e);
                            let x = self.do_bind2_id(core.get_type().clone(), result.get_type().clone());
                            Apply(box Apply(box x, box core), box result)
                        }
                        module::DoBinding::DoBind(pattern, e) => {
                            let e2 = self.translate_expr(e);
                            self.do_bind_translate(pattern.node, e2, result)
                        }
                        module::DoBinding::DoLet(bs) => {
                            Let(self.translate_bindings(bs), box result)
                        }
                    };
                }
                result
            }
            module::Expr::TypeSig(expr, _) => self.translate_expr(*expr),
            module::Expr::Paren(expr) => self.translate_expr(*expr)
        }
    }
    ///Translates
    ///do { expr; stmts } = expr >> do { stmts; }
    fn do_bind2_id(&mut self, m_a: Type, m_b: Type) -> Expr<Id<Name>> {
        debug!("m_a {:?}", m_a);
        let c = match *m_a.appl() {
            Type::Variable(ref var) => vec![Constraint { class: Name { name: intern("Monad"), uid: 0 }, variables: vec![var.clone()] }],
            _ => vec![]
        };
        let typ = function_type_(m_a, function_type_(m_b.clone(), m_b));
        Identifier(Id::new(Name { name: intern(">>"), uid: 0}, typ, c))
    } 
    ///Translates
    ///do {p <- e; stmts} 	=
    ///    let ok p = do {stmts}
	///        ok _ = fail "..."
	///    in e >>= ok
    fn do_bind_translate(&mut self, pattern: module::Pattern<Name>, expr: Expr<Id<Name>>, result: Expr<Id<Name>>) -> Expr<Id<Name>> {

        let m_a = expr.get_type().clone();
        let a = m_a.appr().clone();
        let m_b = result.get_type().clone();
                debug!("m_a {:?}", m_a);
        let c = match *m_a.appl() {
            Type::Variable(ref var) => vec![Constraint { class: Name { name: intern("Monad"), uid: 0 }, variables: vec![var.clone()] }],
            _ => vec![]
        };
        let arg2_type = function_type_(a.clone(), m_b.clone());
        let bind_typ = function_type_(m_a, function_type_(arg2_type.clone(), m_b.clone()));
        let bind_ident = Identifier(Id::new(Name { name: intern(">>="), uid: 0}, bind_typ, c.clone()));

        //Create ok binding
        let func_ident = Id::new(
            self.name_supply.from_str("#ok"),
            arg2_type.clone(),
            c.clone()
        );//TODO unique id
        let var = Id::new(self.name_supply.from_str("p"), function_type_(a, m_b.clone()), c.clone());//Constraints for a
        let fail_ident = Identifier(Id::new(Name { name: intern("fail"), uid: 0 }, function_type_(list_type(char_type()), m_b), c));
        let func = Lambda(var.clone(), box Case(box Identifier(var), 
            vec![Alternative { pattern: self.translate_pattern(pattern), expression: result }
            , Alternative { pattern: Pattern::WildCard, expression: Apply(box fail_ident, box string("Unmatched pattern in let")) } ]));
        let bind = Binding { name: func_ident.clone(), expression: func };
        
        Let(vec![bind], box apply(bind_ident, (vec![expr, Identifier(func_ident)]).into_iter()))
    }

    fn translate_bindings(&mut self, bindings: Vec<module::Binding<Name>>) -> Vec<Binding<Id<Name>>> {
        let mut result = Vec::new();
        let mut vec: Vec<module::Binding<Name>> = Vec::new();
        for bind in bindings.into_iter() {
            if vec.len() > 0 && vec.get(0).name != bind.name {
                result.push(self.translate_matching_groups(vec));
                vec = Vec::new();
            }
            vec.push(bind);
        }
        if vec.len() > 0 {
            result.push(self.translate_matching_groups(vec));
        }
        result
    }
    
    fn unwrap_pattern(&mut self, uid: uint, id: Id<Name>, pattern: module::Pattern<Name>, result: &mut Vec<(Id<Name>, Pattern<Id<Name>>)>) {
        match pattern {
            module::Pattern::Constructor(ctor_name, mut patterns) => {
                let index = result.len();
                let mut name = ::std::string::String::from_str(id.name.name.as_slice());
                let base_length = name.len();
                result.push((id, Pattern::Number(0)));//Dummy
                for (i, p) in patterns.iter_mut().enumerate() {
                    let x = match *p {
                        module::Pattern::Constructor(..) | module::Pattern::Number(..) => {
                            //HACK, by making the variable have the same uid as
                            //the index the newly generated pattern will be recognized
                            //as the same since their binding variable are the same
                            name.truncate(base_length);
                            name.push_char('_');
                            name.push_str(i.to_str().as_slice());

                            let n = Name { name: intern(name.as_slice()), uid: uid };
                            Some(module::Pattern::Identifier(n))
                        }
                        _ => None
                    };
                    match x {
                        Some(mut x) => {
                            ::std::mem::swap(p, &mut x);
                            let id = match *p {
                                module::Pattern::Identifier(ref n) => Id::new(n.clone(), Type::new_var(intern("a")), vec![]),
                                _ => panic!()
                            };
                            self.unwrap_pattern(uid, id, x, result);
                        }
                        None => ()
                    }
                }
                *result.get_mut(index).mut1() = self.translate_pattern(module::Pattern::Constructor(ctor_name, patterns));
            }
            _ => result.push((id, self.translate_pattern(pattern)))
        }
    }
    ///Translates a pattern list of patterns into a list of patterns which are not nested.
    ///The first argument of each tuple is the identifier that is expected to be passed to the case.
    fn unwrap_patterns(&mut self, uid: uint, arg_ids: &[Id<Name>], arguments: &[module::Pattern<Name>]) -> Vec<(Id<Name>, Pattern<Id<Name>>)> {
        let mut result = Vec::new();
        for (p, id) in arguments.iter().zip(arg_ids.iter()) {
            self.unwrap_pattern(uid, id.clone(), p.clone(), &mut result);
        }
        result
    }

    ///Translates a case expression into the core language.
    ///Since the core language do not have nested patterns the patterns are unwrapped into
    ///multiple case expressions.
    fn translate_case(&mut self, expr: module::TypedExpr<Name>, alts: Vec<module::Alternative<Name>>) -> Expr<Id<Name>> {
        let mut vec = Vec::new();
        let dummy_var = [Id::new(self.name_supply.anonymous(), Type::new_var(intern("a")), vec![])];
        let uid = self.name_supply.next_id();
        for module::Alternative { pattern: pattern, matches: matches, where_bindings: where_bindings } in alts.into_iter() {
            let bindings = where_bindings.map_or(Vec::new(), |bs| self.translate_bindings(bs));
            vec.push((self.unwrap_patterns(uid, dummy_var, [pattern.node]), bindings, matches));
        }
        let mut x = self.translate_equations_(vec);
        match x {
            Case(ref mut body, _) => {
                **body = self.translate_expr(expr);
            }
            _ => panic!("Not case")
        }
        x
    }
    ///Translates a binding group such as
    ///map f (x:xs) = e1
    ///map f [] = e2
    fn translate_matching_groups(&mut self, mut bindings: Vec<module::Binding<Name>>) -> Binding<Id<Name>> {
        //If the binding group is simple (no patterns and only one binding)
        //then we do a simple translation to preserve the names for the arguments.
        if bindings.len() == 1 && simple_binding(bindings.get(0)) {
            let module::Binding {
                name: name,
                arguments: arguments, matches: matches,
                typ: module::Qualified { constraints: constraints, value: typ, },
                where_bindings: where_bindings
            } = bindings.pop().unwrap();
            let arg_iterator = arguments.into_iter().map(|p| {
                match p {
                    module::Pattern::Identifier(n) => n,
                    module::Pattern::WildCard => Name { name: intern("_"), uid: -1 },
                    _ => panic!("simple_binding fail")
                }
            });
            let expr = {
                let lambda_ids = lambda_iterator(&typ)
                    .zip(arg_iterator)
                    .map(|(typ, arg)| {
                    Id::new(arg, typ.clone(), vec![])
                });
                let where_bindings_binds = where_bindings.map_or(Vec::new(), |bs| self.translate_bindings(bs).into_iter().collect());
                make_lambda(lambda_ids, make_let(where_bindings_binds, self.translate_match(matches)))
            };
            return Binding {
                name: Id::new(name, typ, constraints),
                expression: expr
            }
        }
        //Generate new names for each of the arguments (since it is likely that not all arguments have a name)
        let mut arg_ids = Vec::new();
        let name;
        {
            let binding0 = bindings.get(0);
            name = Id::new(binding0.name.clone(), binding0.typ.value.clone(), binding0.typ.constraints.clone());
            let mut typ = &binding0.typ.value;
            for _ in range(0, binding0.arguments.len()) {
                arg_ids.push(Id::new(self.name_supply.from_str("arg"), typ.clone(), vec![]));
                typ = match *typ {
                    Type::Application(_, ref next) => &**next,
                    _ => typ//We dont actually have a function type which we need, so we are likely in a unittest
                            //just reuse the same type so we do not crash
                };
            }
        }
        //First we flatten all the patterns that occur in each equation
        //(2:xs) -> [(x:xs), 2]
        let uid = self.name_supply.next_id();
        let equations: Vec<_> = bindings.into_iter().map(|bind| {
            let module::Binding {
                arguments: arguments,
                matches: matches,
                where_bindings: where_bindings,
                ..
            } = bind;
            let where_bindings_binds = where_bindings.map_or(Vec::new(), |bs| self.translate_bindings(bs));
            (self.unwrap_patterns(uid, arg_ids.as_slice(), arguments), where_bindings_binds, matches)
        }).collect();
        let mut expr = self.translate_equations_(equations);
        expr = make_lambda(arg_ids.into_iter(), expr);
        debug!("Desugared {:?} :: {:?}\n {:?}", name.name, name.typ, expr);
        Binding {
            name: name,
            expression: expr
        }
    }
    fn translate_equations_(&mut self, equations: Vec<(Vec<(Id<Name>, Pattern<Id<Name>>)>, Vec<Binding<Id<Name>>>, module::Match<Name>)>) -> Expr<Id<Name>> {
        let mut eqs: Vec<Equation> = Vec::new();
        for &(ref ps, ref bs, ref e) in equations.iter() {
            eqs.push(Equation(ps.as_slice(), (bs.as_slice(), e)));
        }
        for e in eqs.iter() {
            debug!("{:?}", e);
        }
        self.translate_equations(eqs.as_slice())
    }

    ///Translates a list of guards, if no guards matches then the result argument will be the result
    fn translate_guards(&mut self, mut result: Expr<Id<Name>>, guards: &[module::Guard<Name>]) -> Expr<Id<Name>> {
        for guard in guards.iter().rev() {
            let predicate = box self.translate_expr(guard.predicate.clone());
            result = Case(predicate, vec![
                Alternative { pattern: bool_pattern("True"), expression: self.translate_expr(guard.expression.clone()) },
                Alternative { pattern: bool_pattern("False"), expression: result },
            ]);
        }
        result
    }

    fn translate_equations(&mut self, equations: &[Equation]) -> Expr<Id<Name>> {
        ///Returns true if the two patterns would match for the same values
        fn matching<T: PartialEq>(lhs: &(T, Pattern<T>), rhs: &(T, Pattern<T>)) -> bool {
            if lhs.ref0() != rhs.ref0() {
                return false;
            }
            match (lhs.ref1(), rhs.ref1()) {
                (&Pattern::Constructor(ref l, _), &Pattern::Constructor(ref r, _)) => *l == *r,
                (&Pattern::Constructor(..), &Pattern::Number(..)) => false,
                (&Pattern::Number(..), &Pattern::Constructor(..)) => false,
                _ => true
            }
        }
        debug!("In {:?}", equations);
        let &Equation(ps, (where_bindings_bindings, e)) = &equations[0];
        if ps.len() == 0 {
            assert_eq!(equations.len(), 1);//Otherwise multiple matches for this group
            let bindings = where_bindings_bindings.iter().map(|x| x.clone()).collect();
            return make_let(bindings, self.translate_match((*e).clone()));
        }
        if ps.len() == 1 {
            let mut alts: Vec<Alternative<Id<Name>>> = Vec::new();
            for (i, &Equation(ps, (where_bindings_bindings, m))) in equations.iter().enumerate() {
                let bindings = where_bindings_bindings.iter().map(|x| x.clone()).collect();
                match *m {
                    module::Match::Simple(ref e) => {
                        let alt = if ps.len() == 0 {
                            Alternative {
                                pattern: Pattern::WildCard, expression:
                                make_let(bindings, self.translate_expr((*e).clone()))
                            }
                        }
                        else {
                            Alternative {
                                pattern: ps[0].ref1().clone(),
                                expression: make_let(bindings, self.translate_expr((*e).clone()))
                            }
                        };
                        alts.push(alt);
                    }
                    module::Match::Guards(ref guards) => {
                        let fallthrough = if equations.len() == i + 1 {
                            unmatched_guard()
                        }
                        else {
                            self.translate_equations(equations.slice_from(i + 1))
                        };
                        alts.push(Alternative {
                            pattern: ps[0].ref1().clone(),
                            expression: make_let(bindings, self.translate_guards(fallthrough, *guards))
                        });
                    }
                }
            }
            let body = box Identifier(ps[0].ref0().clone());
            return Case(body, alts);
        }
        
        let mut last_index = 0;
        let mut vec: Vec<Equation> = Vec::new();
        let mut alts: Vec<Alternative<Id<Name>>> = Vec::new();
        let mut visited = Vec::new();
        loop {
            //Find the first pattern which does a test and is not already used
            let mut pattern_test = None;
            while last_index < equations.len() {
                let &Equation(ps, _) = &equations[last_index];
                if ps.len() > 0  {
                    match *ps[0].ref1() {
                        Pattern::Constructor(..) | Pattern::Number(..)
                        if visited.iter().find(|x| matching(**x, &ps[0])).is_none() => {
                            pattern_test = Some(&ps[0]);
                            visited.push(&ps[0]);
                            last_index += 1;
                            break;
                        }
                        _ => ()
                    }
                }
                last_index += 1;
            }
            match pattern_test {
                Some(pattern_test) => {
                    vec.clear();
                    let mut variable_bindings = Vec::new();
                    //Gather all patterns which matches the pattern
                    for &Equation(patterns, expr) in equations.iter() {
                        if patterns.len() > 0 && matching(pattern_test, &patterns[0]) {
                            vec.push(Equation(patterns.slice_from(1), expr));
                            //If the patter_test is a constructor we need to add the variables
                            //of the other patterns in a let binding to make sure that all names exist
                            match (patterns[0].ref1(), pattern_test.ref1()) {
                                (&Pattern::Constructor(_, ref l_vars), &Pattern::Constructor(_, ref r_vars)) => {
                                    for (l_var, r_var) in l_vars.iter().zip(r_vars.iter()) {
                                        if l_var != r_var {
                                            variable_bindings.push(Binding { name: l_var.clone(), expression: Identifier(r_var.clone()) });
                                        }
                                    }
                                }
                                _ => ()
                            }
                        }
                        else if patterns.len() == 0 {
                            vec.push(Equation(patterns, expr));
                        }
                    }
                    //For all the pattern that match the pattern we need to generate new case expressions
                    let e = make_let(variable_bindings, self.translate_equations(vec.as_slice()));

                    let arg_id = ps[0].ref0();
                    let bs = needed_variables(arg_id, equations);
                    alts.push(Alternative {
                        pattern: pattern_test.ref1().clone(),
                        expression: make_let(bs, e)
                    });
                }
                None => break
            }
        }
        if alts.len() == 0 {
            for &Equation(patterns, expr) in equations.iter() {
                vec.push(Equation(patterns.slice_from(1), expr));
            }
            let &Equation(ps, _) = &equations[0];
            let arg_id = ps[0].ref0();
            let bs = needed_variables(arg_id, equations);
            make_let(bs, self.translate_equations(vec.as_slice()))
        }
        else {
            let defaults: Vec<Equation> = equations.iter()
                .filter(|& &Equation(ps, _)| ps.len() > 0 && (match *ps[0].ref1() { Pattern::WildCard | Pattern::Identifier(..) => true, _ => false }))
                .map(|&Equation(ps, e)| Equation(ps.slice_from(1), e))
                .collect();
            if defaults.len() != 0 {
                let arg_id = ps[0].ref0();
                let bs = needed_variables(arg_id, equations);
                let e = make_let(bs, self.translate_equations(defaults.as_slice()));
                alts.push(Alternative {
                    pattern: Pattern::WildCard,
                    expression: e
                });
            }
            let &Equation(ps, _) = &equations[0];
            let body = box Identifier(ps[0].ref0().clone());
            Case(body, alts)
        }
    }

    fn translate_pattern(&mut self, pattern: module::Pattern<Name>) -> Pattern<Id<Name>> {
        match pattern {
            module::Pattern::Identifier(i) => Pattern::Identifier(Id::new(i, Type::new_var(intern("a")), vec![])),
            module::Pattern::Number(n) => Pattern::Number(n),
            module::Pattern::Constructor(name, patterns) => {
                let ps = patterns.into_iter().map(|pat| {
                    match pat {
                        module::Pattern::Identifier(name) => Id::new(name, Type::new_var(intern("a")), vec![]),
                        module::Pattern::WildCard => Id::new(Name { name: intern("_"), uid: -1 }, Type::new_var(intern("a")), vec![]),
                        _ => panic!("Nested pattern")
                    }
                }).collect();
                Pattern::Constructor(Id::new(name, Type::new_var(intern("a")), vec![]), ps)
            }
            module::Pattern::WildCard => Pattern::WildCard
        }
    }
}

    fn bool_pattern(s: &str) -> Pattern<Id<Name>> {
        Pattern::Constructor(Id::new(Name { name: intern(s), uid: 0 }, bool_type(), vec![]), vec![])
    }

    struct LambdaIterator<'a> {
        typ: &'a Type
    }
    impl <'a> Iterator for LambdaIterator<'a> {
        type Item = &'a Type;
        fn next(&mut self) -> Option<&'a Type> {
            match *self.typ {
                Type::Application(ref lhs, ref rhs) => {
                    match **lhs {
                        Type::Application(ref func, _) => {
                            match **func {
                                Type::Constructor(ref op) if op.name.as_slice() == "->" => {
                                    let func = self.typ;
                                    self.typ = &**rhs;
                                    Some(func)
                                }
                                _ => None
                            }
                        }
                        _ => None
                    }
                }
                _ => None
            }
        }
    }
    //Creates an iterator which walks through all the function types that are needed
    //when creating a lambda with make_lambda
    //Ex: (a -> b -> c) generates [(a -> b -> c), (b -> c)]
    fn lambda_iterator<'a>(typ: &'a Type) -> LambdaIterator<'a> {
        LambdaIterator { typ: typ }
    }
    ///Tests that the binding has no patterns for its arguments
    fn simple_binding(binding: &module::Binding<Name>) -> bool {
        binding.arguments.iter().all(|arg| {
            match *arg {
                module::Pattern::WildCard | module::Pattern::Identifier(..) => true,
                _ => false
            }
        })
    }

    ///Creates a lambda from an iterator of its arguments and body
    fn make_lambda<T, I: Iterator<Item=T>>(mut iter: I, body: Expr<T>) -> Expr<T> {
        match iter.next() {
            Some(arg) => Lambda(arg, box make_lambda(iter, body)),
            None => body
        }
    }
    ///Creates a function application from a function and its arguments
    fn apply<T, I: Iterator<Item=Expr<T>>>(mut func: Expr<T>, mut iter: I) -> Expr<T> {
        for arg in iter {
            func = Apply(box func, box arg);
        }
        func
    }
    ///Creates a let binding, but if there is no bindings the let is omitted
    fn make_let<T>(bindings: Vec<Binding<T>>, expr: Expr<T>) -> Expr<T> {
        if bindings.len() == 0 {
            expr
        }
        else {
            Let(bindings, box expr)
        }
    }

    ///Takes a id of the variable passed to the case and returns a vector
    ///of bindings which need to be added to make sure no variables are missing
    fn needed_variables(arg_id: &Id<Name>, equations: &[Equation]) -> Vec<Binding<Id<Name>>> {
        equations.iter()
            .filter(|& &Equation(ps, _)| ps.len() > 0 && (match *ps[0].ref1() { Pattern::WildCard | Pattern::Identifier(..) => true, _ => false }))
            .map(|eq| {
            let &Equation(ps, _) = eq;
            let other_id = match *ps[0].ref1() {
                Pattern::Identifier(ref name) => name.clone(),
                Pattern::WildCard => Id::new(Name { name: intern("_"), uid: -1 }, Type::new_var(intern("a")), vec![]),
                _ => panic!()
            };
            Binding { name: other_id, expression: Identifier(arg_id.clone()) }
        }).collect()
    }
    ///Creates a string literal expressions from a &str
    fn string(s: &str) -> Expr<Id<Name>> {
        Literal(Literal { typ: list_type(char_type()), value: String(intern(s)) })
    }
    ///Creates an expression which reports an unmatched guard error when executed
    fn unmatched_guard() -> Expr<Id<Name>> {
        let error_ident = Identifier(Id::new(Name { name: intern("error"), uid: 0 }, function_type_(list_type(char_type()), Type::new_var(intern("a"))), vec![]));
        Apply(box error_ident, box string("Unmatched guard"))
    }

}
