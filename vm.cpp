// Source: https://usaco.guide/general/io

#include <iostream>
#include <fstream>
#include <vector>
#include <string>
#include <unordered_map>
#include <stdexcept>
#include <nlohmann/json.hpp>
using namespace std;
using json = nlohmann::json;

enum class OP{
    Add, Sub, Mul, Div, Gt, Lt, Eq, Ne, Asig, Print,
    PrintStr, Return, Goto, GotoF, Era, Param, GoSub,
    EndFunc};

struct VmQuad{
    OP op;
    string arg1, arg2, result;
};

struct VmConstants{
    unordered_map<int, long long> ints;
    unordered_map<int, long double> floats;
};

struct GlobalSizes{
    int n_int, n_float;
};

struct FuncMeta{
    int n_local_int, n_local_float, n_temp_int, n_temp_float;
};

struct MainMeta{
    int n_temp_int, n_temp_float;
};


struct VmPayload{
    vector<VmQuad> quads;
    VmConstants constants;
    GlobalSizes globals;
    unordered_map<string, FuncMeta> funcs;
    MainMeta main;
};

struct ActivationRecord{
    vector<long long> local_int;
    vector<long double> local_float;
    vector<long long> temp_int;
    vector<long double> temp_float;
};

struct VmState{
    vector<ActivationRecord> call_stack;
    vector<long long> global_int;
    vector<long double> global_float;
    vector<long long> const_int;
    vector<long double> const_float;
    vector<ActivationRecord> pending_ar_stack; // Es el activation record que construimos cuando vamos a llamar a una funcion
                                 // Tiene que ser una stack porque aveces llamamos una funcion dentro de la llamada de una funcion
    vector<long long> ip_stack; // Para manejar a donde tienen que volver las funciones

    VmState(const VmPayload& payload) {
        // Llenamos la memoria global de la vm (globales mas constantes)
        global_int = vector<long long>(payload.globals.n_int, 0);
        global_float = vector<long double> (payload.globals.n_float, 0.0);
        const_int = vector<long long> (payload.constants.ints.size(), 0);
        const_float = vector<long double> (payload.constants.floats.size(), 0.0);

        // Llenamos const int y const float
        for(auto c: payload.constants.ints){
            const_int[c.first - 18000] = c.second;
        }
        for(auto c: payload.constants.floats){
            const_float[c.first - 19000] = c.second;
        }

        // Llenamos primer activation record con informacion de main
        vector<long long> local_int = vector<long long>(0);
        vector<long double> local_float = vector<long double>(0);
        vector<long long> temp_int = vector<long long>(payload.main.n_temp_int, 0);
        vector<long double> temp_float = vector<long double>(payload.main.n_temp_float, 0.0);
        call_stack.push_back(ActivationRecord({local_int, local_float, temp_int, temp_float}));
    }
    VmState(){
        // default
    }
};

struct QuadSolver{
    VmPayload payload;
    VmState state;

    QuadSolver(const VmPayload &_payload){
        payload = _payload;
        state= VmState(payload);
    }

    bool is_float(int addr){
        if(addr >= 2000 && addr <= 2999)return true; // Flotante global
        if(addr >= 6000 && addr <= 6999)return true; // Flotante Local
        if(addr >= 14000 && addr <= 14999)return true; // Flotante Temporal
        if(addr >= 19000 && addr <= 19999)return true; // Flotante Constante
        return false; // No es flotante
    }
    long long read_int(int addr){
        if (addr >= 18000) return state.const_int[addr - 18000]; // Entero Constante
        if (addr >= 13000) return state.call_stack.back().temp_int[addr - 13000]; // Entero Temporal
        if (addr >= 5000)  return state.call_stack.back().local_int[addr - 5000]; // Entero Local
        if (addr >= 1000)  return state.global_int[addr - 1000]; // Entero Global
        throw runtime_error("read_int: dir inválida " + to_string(addr));
    }
    long double read_float(int addr){
        if (addr >= 19000) return state.const_float[addr - 19000]; // Flotante Constante
        if (addr >= 14000) return state.call_stack.back().temp_float[addr - 14000]; // Flotante Temporal
        if (addr >= 6000)  return state.call_stack.back().local_float[addr - 6000]; // Flotante Local
        if (addr >= 2000)  return state.global_float[addr - 2000]; // Flotante Global
        throw runtime_error("read_float: dir inválida " + to_string(addr));
    }

    void write_int(int addr, long long val){
        if (addr >= 13000) state.call_stack.back().temp_int[addr - 13000] = val; // Entero Temporal
        else if (addr >= 5000)  state.call_stack.back().local_int[addr - 5000] = val; // Entero Local
        else if (addr >= 1000)  state.global_int[addr - 1000] = val; // Entero Global
        else throw runtime_error("write_int: dir inválida " + to_string(addr));
        // Entero constante es read only
    }

    void write_float(int addr, long double val){
        if (addr >= 14000) state.call_stack.back().temp_float[addr - 14000] = val; // Flotante Temporal
        else if (addr >= 6000)  state.call_stack.back().local_float[addr - 6000] = val; // Flotante Local
        else if (addr >= 2000)  state.global_float[addr - 2000] = val; // Flotante Global
        else throw runtime_error("write_float: dir inválida " + to_string(addr));
        // Flotante constante es read only
    }
    void solve(){
        int quad_idx = 0;
        while(quad_idx < (int)payload.quads.size()){
            solveQuad(quad_idx);
        }
    }

    void solveQuad(int &quad_idx){
        VmQuad currQuad = payload.quads[quad_idx];
        switch(currQuad.op){
            case OP::Add:{
                int arg1 = stoi(currQuad.arg1), arg2 = stoi(currQuad.arg2), result = stoi(currQuad.result);
                long double val1 = is_float(arg1) ? read_float(arg1): read_int(arg1);
                long double val2 = is_float(arg2) ? read_float(arg2): read_int(arg2);
                // Este tipo de if se hace por errores de precision que podemos evitar cuando lideamos con 2 enteros
                if(is_float(result)){
                    write_float(result, val1 + val2);
                }else{
                    write_int(result, read_int(arg1) + read_int(arg2));
                }
                break;
            }
            case OP::Sub:{
                int arg1 = stoi(currQuad.arg1), arg2 = stoi(currQuad.arg2), result = stoi(currQuad.result);
                long double val1 = is_float(arg1) ? read_float(arg1): read_int(arg1);
                long double val2 = is_float(arg2) ? read_float(arg2): read_int(arg2);
                if(is_float(result)){
                    write_float(result, val1 - val2);
                }else{
                    write_int(result, read_int(arg1) - read_int(arg2));
                }
                break;
            }
            case OP::Mul:{
                int arg1 = stoi(currQuad.arg1), arg2 = stoi(currQuad.arg2), result = stoi(currQuad.result);
                long double val1 = is_float(arg1) ? read_float(arg1): read_int(arg1);
                long double val2 = is_float(arg2) ? read_float(arg2): read_int(arg2);
                if(is_float(result)){
                    write_float(result, val1 * val2);
                }else{
                    write_int(result, read_int(arg1) * read_int(arg2));
                }
                break;

            }
            case OP::Div:{
                int arg1 = stoi(currQuad.arg1), arg2 = stoi(currQuad.arg2), result = stoi(currQuad.result);
                long double val1 = is_float(arg1) ? read_float(arg1): read_int(arg1);
                long double val2 = is_float(arg2) ? read_float(arg2): read_int(arg2);
                if(is_float(result)){
                    write_float(result, val1 / val2);
                }else{
                    write_int(result, read_int(arg1) / read_int(arg2));
                }
                break;
            }
            case OP::Gt:{
                int arg1 = stoi(currQuad.arg1), arg2 = stoi(currQuad.arg2), result = stoi(currQuad.result);
                long long res;
                if(!is_float(arg1) && !is_float(arg2)){
                    res = (read_int(arg1) > read_int(arg2)) ? 1: 0;
                }else{
                    long double a = is_float(arg1) ? read_float(arg1) : (long double)read_int(arg1);
                    long double b = is_float(arg2) ? read_float(arg2) : (long double)read_int(arg2);
                    res = (a > b) ? 1 : 0;
                }
                write_int(result, res);
                break;
            }
            case OP::Lt:{
                int arg1 = stoi(currQuad.arg1), arg2 = stoi(currQuad.arg2), result = stoi(currQuad.result);
                long long res;
                if(!is_float(arg1) && !is_float(arg2)){
                    res = (read_int(arg1) < read_int(arg2)) ? 1: 0;
                }else{
                    long double a = is_float(arg1) ? read_float(arg1) : (long double)read_int(arg1);
                    long double b = is_float(arg2) ? read_float(arg2) : (long double)read_int(arg2);
                    res = (a < b) ? 1 : 0;
                }
                write_int(result, res);
                break;
            }
            case OP::Eq:{
                int arg1 = stoi(currQuad.arg1), arg2 = stoi(currQuad.arg2), result = stoi(currQuad.result);
                long long res;
                if(!is_float(arg1) && !is_float(arg2)){
                    res = (read_int(arg1) == read_int(arg2)) ? 1: 0;
                }else{
                    long double a = is_float(arg1) ? read_float(arg1) : (long double)read_int(arg1);
                    long double b = is_float(arg2) ? read_float(arg2) : (long double)read_int(arg2);
                    res = (a == b) ? 1 : 0;
                }
                write_int(result, res);
                break;

            }
            case OP::Ne:{
                int arg1 = stoi(currQuad.arg1), arg2 = stoi(currQuad.arg2), result = stoi(currQuad.result);
                long long res;
                if(!is_float(arg1) && !is_float(arg2)){
                    res = (read_int(arg1) != read_int(arg2)) ? 1: 0;
                }else{
                    long double a = is_float(arg1) ? read_float(arg1) : (long double)read_int(arg1);
                    long double b = is_float(arg2) ? read_float(arg2) : (long double)read_int(arg2);
                    res = (a != b) ? 1 : 0;
                }
                write_int(result, res);
                break;

            }
            case OP::Asig:{
                int arg1 = stoi(currQuad.arg1), result = stoi(currQuad.result);
                if(is_float(result)){
                    long double a = is_float(arg1) ? read_float(arg1) : read_int(arg1);
                    write_float(result, a);
                }else{
                    write_int(result, read_int(arg1));
                }
                break;
            }
            case OP::Print:{
                int arg1 = stoi(currQuad.arg1);
                if(is_float(arg1)){
                    cout << read_float(arg1) << endl;
                }else{
                    cout << read_int(arg1) << endl;
                }
                break;
            }
            case OP::PrintStr:{
                string s = currQuad.arg1;
                // Quitamos las ""
                s = s.substr(1, s.size() - 2);
                cout << s << endl;
                break;
            }
            case OP::Return:{
                int arg1 = stoi(currQuad.arg1), result = stoi(currQuad.result);
                if(is_float(result)){
                    long double a = is_float(arg1) ? read_float(arg1) : read_int(arg1);
                    state.global_float[result - 2000] = a;
                }else{
                    state.global_int[result - 1000] = read_int(arg1);
                }

                // Borramos act record, y volvemos a donde llamamos la funcion
                state.call_stack.pop_back();
                quad_idx = state.ip_stack.back();
                state.ip_stack.pop_back();
                return;
                break;
            }
            case OP::Goto:{
                quad_idx = stoi(currQuad.result);
                return;
                break;
            }
            case OP::GotoF:{
                int arg1 = stoi(currQuad.arg1);
                if(read_int(arg1) == 0){
                    quad_idx = stoi(currQuad.result);
                    return;
                }
                break;
            }
            case OP::Era:{
                string func_name = currQuad.arg1;
                // Llenamos el pending activation record de la funcion que vamos a llamar, asi reservando su memoria
                vector<long long> local_int = vector<long long>(payload.funcs[func_name].n_local_int, 0);
                vector<long double> local_float = vector<long double>(payload.funcs[func_name].n_local_float, 0.0);
                vector<long long> temp_int = vector<long long>(payload.funcs[func_name].n_temp_int, 0);
                vector<long double> temp_float = vector<long double>(payload.funcs[func_name].n_temp_float, 0.0);
                state.pending_ar_stack.push_back(ActivationRecord({local_int, local_float, temp_int, temp_float}));
                break;
            }
            case OP::Param:{
                // Metemos valor en memoria del pending activation record
                int arg1 = stoi(currQuad.arg1), result = stoi(currQuad.result);
                if(is_float(result)){
                    long double a = is_float(arg1) ? read_float(arg1) : read_int(arg1);
                    state.pending_ar_stack.back().local_float[result - 6000] = a;
                }else{
                    state.pending_ar_stack.back().local_int[result - 5000] = read_int(arg1);
                }
                break;
            }

            case OP::GoSub:{
                // Guardamos lugar al que tenemos que regresar
                // Nos vamos al cuadruplo inicial de la funcion
                // Activamos el activation record
                state.ip_stack.push_back(quad_idx + 1);
                state.call_stack.push_back(state.pending_ar_stack.back());
                state.pending_ar_stack.pop_back();
                quad_idx = stoi(currQuad.result);
                return;
                break;
            }
            case OP::EndFunc:{
                // Borramos act record, y volvemos a donde llamamos la funcion
                state.call_stack.pop_back();
                quad_idx = state.ip_stack.back();
                state.ip_stack.pop_back();
                return;
                break;
            }
            default:
                cout << "shouldnt happen" << endl;
                break;
        }

        quad_idx++;
    }
};

// ─── Convierte el string de op del JSON a OP enum ─────────────────────────
OP parse_op(const string& s) {
    if (s == "+")  return OP::Add;
    if (s == "-")  return OP::Sub;
    if (s == "*")  return OP::Mul;
    if (s == "/")  return OP::Div;
    if (s == ">")  return OP::Gt;
    if (s == "<")  return OP::Lt;
    if (s == "==") return OP::Eq;
    if (s == "!=") return OP::Ne;
    if (s == "=")  return OP::Asig;
    if (s == "PRINT")     return OP::Print;
    if (s == "PRINT_STR") return OP::PrintStr;
    if (s == "RETURN")    return OP::Return;
    if (s == "GOTO")      return OP::Goto;
    if (s == "GOTOF")     return OP::GotoF;
    if (s == "ERA")       return OP::Era;
    if (s == "PARAM")     return OP::Param;
    if (s == "GOSUB")     return OP::GoSub;
    if (s == "ENDFUNC")   return OP::EndFunc;
    throw runtime_error("Opcode desconocido: '" + s + "'");
}

// ─── Lee el .obj (JSON) y construye el VmPayload ──────────────────────────
VmPayload load_from_json(const string& path) {
    ifstream f(path);
    if (!f.is_open()) {
        throw runtime_error("No se pudo abrir el archivo: " + path);
    }

    json j;
    f >> j;

    VmPayload p;

    // 1. Cuádruplos
    for (auto& q : j["quads"]) {
        VmQuad vq;
        vq.op     = parse_op(q["op"].get<string>());
        vq.arg1   = q["arg1"].get<string>();
        vq.arg2   = q["arg2"].get<string>();
        vq.result = q["result"].get<string>();
        p.quads.push_back(vq);
    }

    // 2. Constantes (las llaves del JSON son strings → stoi para volverlas int)
    for (auto& [addr_str, val] : j["constants"]["ints"].items()) {
        p.constants.ints[stoi(addr_str)] = val.get<long long>();
    }
    for (auto& [addr_str, val] : j["constants"]["floats"].items()) {
        p.constants.floats[stoi(addr_str)] = val.get<long double>();
    }

    // 3. Globales
    p.globals.n_int   = j["globals"]["n_int"].get<int>();
    p.globals.n_float = j["globals"]["n_float"].get<int>();

    // 4. Funciones
    for (auto& [name, fm] : j["funcs"].items()) {
        FuncMeta meta;
        meta.n_local_int   = fm["n_local_int"].get<int>();
        meta.n_local_float = fm["n_local_float"].get<int>();
        meta.n_temp_int    = fm["n_temp_int"].get<int>();
        meta.n_temp_float  = fm["n_temp_float"].get<int>();
        p.funcs[name] = meta;
    }

    // 5. Main meta
    p.main.n_temp_int   = j["main"]["n_temp_int"].get<int>();
    p.main.n_temp_float = j["main"]["n_temp_float"].get<int>();

    return p;
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        cerr << "Uso: " << argv[0] << " <archivo.obj>" << endl;
        return 1;
    }

    try {
        VmPayload payload = load_from_json(argv[1]);
        QuadSolver vm(payload);
        vm.solve();
    } catch (const exception& e) {
        cerr << "Error: " << e.what() << endl;
        return 1;
    }

    return 0;
}
